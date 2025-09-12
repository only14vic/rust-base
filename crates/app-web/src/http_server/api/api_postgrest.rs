use {
    crate::{
        WebConfig,
        ext::{AuthToken, Http, RequestExt}
    },
    actix_http::{Method, StatusCode},
    actix_web::{
        HttpRequest, HttpResponse, HttpResponseBuilder,
        error::{ErrorBadRequest, ErrorNotFound},
        http::header,
        web
    },
    app_base::prelude::*,
    bytes::Bytes,
    reqwest::{
        Client,
        header::{HeaderMap, HeaderName, HeaderValue}
    },
    serde_json::{Value, json},
    sqlx::{Acquire, Pool, Postgres, types::time::OffsetDateTime},
    std::{
        borrow::Cow,
        collections::HashMap,
        ops::Not,
        str::FromStr,
        sync::{Arc, LazyLock},
        time::{Duration, SystemTime, UNIX_EPOCH}
    },
    url::Url
};

const POSTGREST_OPERATORS: &[&str] = &[
    "eq", "gt", "gte", "lt", "lte", "neq", "like", "ilike", "match", "imatch", "in",
    "is", "isdistinct", "fts", "plfts", "phfts", "wfts", "cs", "cd", "ov", "sl", "sr",
    "nxr", "nxl", "adj", "not", "or", "and", "all", "any"
];

const POSTGREST_QUERY_PARAMS: &[&str] = &[
    "select", "order", "offset", "limit", "columns", "on_conflict", "or", "and",
    "not.or", "not.and"
];

const FORCE_GET_TO_POST_PATHS: &[&str] = &[
    "/rpc/login", "/rpc/login_confirm", "/rpc/logout", "/rpc/refresh_token"
];
const REFRESH_TOKEN_PATH: &str = "/rpc/refresh_token";
const DEFAULT_ORDER_BY: &str = "id.desc";

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(10)
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .tcp_nodelay(true)
        .tls_sni(false)
        .http1_only()
        .http1_ignore_invalid_headers_in_responses(true)
        .cookie_store(false)
        .no_hickory_dns()
        .no_gzip()
        .no_brotli()
        .no_proxy()
        .build()
        .unwrap()
});

pub async fn api_postgrest(
    req: HttpRequest,
    payload: Option<Bytes>
) -> Http<HttpResponse> {
    let config = req.config::<WebConfig>();
    let api_proxy_path = config.api.path.trim_end_matches("/");

    let mut url = Url::parse(&config.api.proxy_url)?;
    let mut path = req.match_pattern().unwrap_or(req.path().to_string());

    while let (Some(start), Some(mut end)) = (path.find('{'), path.find('}')) {
        if path.get(end + 1..=end + 1) == Some("/") {
            end += 1;
        }
        path.replace_range(start..=end, "");
    }
    let path = path.trim_end_matches('/');

    if path.starts_with(&[api_proxy_path, "/"].concat()) || path == api_proxy_path {
        let mut path = path.trim_start_matches(api_proxy_path);
        if path.is_empty() {
            path = "/";
        }
        if url.scheme() == "unix" {
            url.set_path(&[url.path(), path].concat());
        } else {
            url.set_path(path);
        }
    } else if url.scheme() == "unix" {
        url.set_path(&[url.path(), path].concat());
    } else {
        url.set_path(path);
    }

    let mut query_params =
        web::Query::<HashMap<Cow<str>, Cow<str>>>::from_query(req.query_string())
            .map(|data| data.into_inner())
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, v)| v.trim().is_empty().not())
            .collect::<HashMap<_, _>>();

    if url.path() == "/" && (Env::is_prod() || query_params.contains_key("time")) {
        let mut time = OffsetDateTime::from_unix_timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs()
                .try_into()?
        )?;

        if query_params.get("time").map(|v| v.as_ref()) == Some("sql") {
            let mut db = req
                .app_data::<Arc<Pool<Postgres>>>()
                .ok_or("There is no Pool<Postgres>")?
                .acquire()
                .await
                .unwrap();

            time = sqlx::query!("select now() as time")
                .fetch_one(db.acquire().await?)
                .await?
                .time
                .unwrap();
        }

        return HttpResponse::Ok()
            .json(json!({"time": time.to_string()}))
            .into_ok();
    }

    if req.query_string().is_empty().not() && url.path().starts_with("/rpc/").not() {
        for (name, value) in query_params.iter_mut() {
            let name = name.to_ascii_lowercase();
            if name.starts_with('_').not()
                && POSTGREST_QUERY_PARAMS.contains(&name.as_str()).not()
                && match value.split_once('.') {
                    Some((op, ..)) => {
                        POSTGREST_OPERATORS
                            .contains(
                                &op.to_ascii_lowercase()
                                    .replace("(all)", "")
                                    .replace("(any)", "")
                                    .as_str()
                            )
                            .not()
                    },
                    None => true
                }
            {
                *value = Cow::from(["eq.", value.as_ref()].concat());
            }
        }
    }

    if req.method() == Method::GET
        && url.path().starts_with("/rpc/").not()
        && query_params.contains_key("select").not()
    {
        query_params.insert(Cow::from("select"), Cow::from("id"));
    }

    url.set_query(Some(
        &query_params
            .iter()
            .filter_map(|(n, v)| n.starts_with("_").not().then_some(format!("{n}={v}")))
            .collect::<Vec<_>>()
            .join("&")
    ));

    let mut body = Value::Object(Default::default());
    if let Some(bytes) = payload
        && bytes.is_empty().not()
    {
        body = serde_json::from_slice(bytes.as_ref())
            .map_err(|_| ErrorBadRequest("Invalid json body."))?;
    }

    if let Some(obj) = body.as_object_mut() {
        for (name, value) in req.match_info().iter() {
            obj.insert(name.into(), value.into());
        }
    }

    let body: Bytes = Bytes::from_iter(serde_json::to_vec(&body)?);

    let method = match req.method() {
        &Method::PATCH if body.is_empty() || *body == *b"{}" => Method::GET,
        &Method::GET if FORCE_GET_TO_POST_PATHS.contains(&url.path()) => Method::POST,
        method => method.clone()
    };

    if [Method::PATCH, Method::DELETE].contains(&method) {
        if query_params.is_empty()
            || query_params
                .iter()
                .any(|(n, ..)| POSTGREST_QUERY_PARAMS.contains(&n.as_ref()).not())
                .not()
        {
            return Err(ErrorBadRequest("Filter arguments required."))?;
        }

        url.query_pairs_mut().append_pair("limit", "1");
        if url.query_pairs().any(|(n, ..)| n == "order").not() {
            url.query_pairs_mut().append_pair("order", DEFAULT_ORDER_BY);
        }
    }

    let mut api_req = HTTP_CLIENT
        .request(reqwest::Method::from_str(method.as_str())?, url.clone())
        .headers(HeaderMap::from_iter(req.headers().iter().filter_map(
            |(n, v)| {
                match (
                    HeaderName::from_str(n.as_str()),
                    HeaderValue::from_bytes(v.as_bytes())
                ) {
                    (Ok(n), Ok(v)) => Some((n, v)),
                    _ => None
                }
            }
        )))
        .fetch_mode_no_cors()
        .build()?;

    if req.headers().contains_key("prefer").not() {
        if [Method::POST, Method::PUT, Method::PATCH, Method::DELETE].contains(&method)
            && api_req.url().path().starts_with("/rpc/").not()
        {
            api_req.headers_mut().insert(
                HeaderName::from_static("prefer"),
                HeaderValue::from_static("return=representation")
            );
        } else if Method::GET == method && api_req.url().path().starts_with("/rpc/").not()
        {
            api_req.headers_mut().insert(
                HeaderName::from_static("prefer"),
                HeaderValue::from_static("count=exact")
            );
        }
    }

    if body.is_empty().not() {
        api_req.headers_mut().insert(
            reqwest::header::CONTENT_LENGTH,
            HeaderValue::from_str(&body.len().to_string())?
        );
        *api_req.body_mut() = Some(body.into());
    }

    let mut set_cookie_headers = Default::default();

    // Pre request to refresh access token
    if let (Some(auth), Some(refresh_token)) = (
        req.headers().get(header::AUTHORIZATION),
        req.cookie("refresh_token")
    ) {
        let auth = String::from_utf8(auth.as_bytes().to_vec()).unwrap_or_default();
        let auth = auth
            .get(auth.find(' ').unwrap_or(0)..)
            .unwrap_or_default()
            .trim_start();

        if [Method::GET, Method::HEAD].contains(&method) && auth == refresh_token.value()
        {
            let mut api_url = Url::parse(&config.api.proxy_url)?;
            api_url.set_path(REFRESH_TOKEN_PATH);

            let mut api_pre_req = HTTP_CLIENT
                .request(reqwest::Method::POST, api_url)
                .headers(api_req.headers().clone())
                .build()?;

            api_pre_req.headers_mut().insert(
                reqwest::header::CONTENT_LENGTH,
                HeaderValue::from_static("0")
            );
            api_pre_req.headers_mut().insert(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            );

            let res = HTTP_CLIENT.execute(api_pre_req).await?;

            set_cookie_headers = res
                .headers()
                .get_all(reqwest::header::SET_COOKIE)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();

            if let Ok(token) = res.json::<AuthToken>().await
                && token.access_token().is_empty().not()
            {
                api_req.headers_mut().insert(
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token.access_token()))?
                );
            }
        }
    }

    let api_res = HTTP_CLIENT.execute(api_req).await?;

    let mut res_builder =
        HttpResponseBuilder::new(StatusCode::from_u16(api_res.status().as_u16())?);

    api_res.headers().into_iter().for_each(|(name, value)| {
        res_builder.append_header((name.as_str(), value.as_bytes()));
    });

    set_cookie_headers.iter().for_each(|v| {
        res_builder.append_header((reqwest::header::SET_COOKIE.as_str(), v.as_bytes()));
    });

    let mut res = None;
    let status = api_res.status();
    let body = api_res.bytes().await?;

    if status.is_client_error() || status.is_server_error() {
        let json = Value::from_json_slice(&body)?;
        res = res_builder.json(json).into();
    } else if [Method::PATCH].contains(&method) && *body == *b"[]" {
        return Err(ErrorNotFound("Data not found."))?;
    }

    if res.is_none() {
        res = res_builder.body(body).into();
    }

    Ok(res.unwrap())
}
