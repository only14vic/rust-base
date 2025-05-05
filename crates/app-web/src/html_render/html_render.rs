use {
    super::{HtmlRenderConfig, HtmlRenderContext},
    crate::ext::{Http, RequestExt},
    actix_web::{
        FromRequest, HttpRequest, HttpResponse,
        dev::Payload,
        error::{ErrorInternalServerError, ErrorNotFound},
        http::header::ContentType
    },
    app_base::prelude::*,
    core::{ops::Deref, pin::Pin},
    futures::future,
    glob::glob,
    regex::Regex,
    std::{
        borrow::Cow,
        collections::HashMap,
        error::Error,
        fmt::Debug,
        fs::read_to_string,
        path::Path,
        sync::{Arc, LazyLock}
    },
    tera::Tera
};

#[derive(Clone)]
pub struct HtmlRender {
    tera: Arc<Tera>,
    config: Arc<HtmlRenderConfig>
}

impl HtmlRender {
    pub fn new(config: &Arc<HtmlRenderConfig>) -> Self {
        let config = config.clone();
        let dir = format!(
            "{}/{}/{}",
            config.assets_dir.trim_end_matches('/'),
            config.default_module.trim_matches('/'),
            config.files_glob.trim_matches('/')
        );

        let mut tera = Tera::parse(&dir).unwrap();

        for (module_name, module_path) in config.modules.iter() {
            let files = Self::find_html_files(
                &config,
                module_name,
                module_path
                    .as_ref()
                    .ok_or(format!(
                        "Empty html render module path for '{module_name}'."
                    ))
                    .unwrap()
                    .as_str()
            )
            .unwrap();
            tera.add_template_files(files).unwrap();
        }

        super::modules::register_modules(&mut tera);

        tera.autoescape_on(vec![]);
        tera.build_inheritance_chains().unwrap();

        let this = Self { tera: tera.into(), config };

        Env::is_debug().then(|| log::trace!("Loaded {this:?}"));

        this
    }

    pub fn add_html_dir(&mut self, module_name: &str, module_path: &str) -> Void {
        let files = Self::find_html_files(&self.config, module_name, module_path)?;
        self.tera.try_mut()?.add_template_files(files)?;
        ok()
    }

    pub fn add_files(&mut self, files: &HashMap<&str, String>) -> Void {
        for (name, file) in files {
            self.tera
                .try_mut()?
                .add_raw_template(name, read_to_string(file)?.as_str())?
        }
        ok()
    }

    fn find_html_files(
        config: &HtmlRenderConfig,
        module_name: &str,
        module_path: &str
    ) -> Ok<Vec<(String, Option<String>)>> {
        let html_pages_dir = config.pages_dir.trim_matches('/');
        let base_path = Path::new(config.assets_dir.trim_end_matches('/'));
        let module_path = base_path.join(module_path.trim_matches('/'));

        if module_path.exists() == false {
            Err(format!(
                "Module path '{}' does not exist for module '{module_name}'.",
                module_path.to_str().unwrap_or_default()
            ))?;
        }

        if module_path.exists() && module_path.is_file() {
            return Ok([(
                module_path.to_string_lossy().into(),
                Some(module_name.into())
            )]
            .to_vec());
        }

        let path = module_path.join(config.files_glob.trim_matches('/'));

        let files = glob(&path.to_string_lossy())?
            .filter_map(|p| p.ok())
            .map(|p| {
                let path = p.to_string_lossy().to_string();
                let mut name =
                    path.replacen(&*module_path.to_string_lossy(), module_name, 1);

                if name.starts_with(&[module_name, "/", html_pages_dir, "/"].concat())
                    && name.contains("/_") == false
                {
                    name = name.replacen(
                        &(module_name.to_owned() + "/" + html_pages_dir + "/"),
                        &(html_pages_dir.to_owned() + "/" + module_name + "/"),
                        1
                    );
                }

                (path, Some(name))
            })
            .collect::<Vec<_>>();

        Ok(files)
    }

    #[inline]
    pub fn render(
        &self,
        template_name: &str,
        context: &HtmlRenderContext
    ) -> tera::Result<String> {
        self.tera.render(template_name, &context.borrow())
    }

    pub async fn render_request(&self, req: &HttpRequest) -> Http<HttpResponse> {
        static CLEAN_REQUEST_PATTERN: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("/?\\{[^\\}]*\\}").unwrap());

        Env::is_debug().then(|| log::trace!("URL: {}", req.path()));

        let path = match req.match_pattern() {
            Some(pattern) => Cow::from(pattern),
            None => Cow::from(req.path())
        };
        let path = CLEAN_REQUEST_PATTERN.replace_all(&path, "");
        let path = urlencoding::decode_binary(path.as_bytes());
        let path = String::from_utf8(path.as_ref().to_vec())?;
        let path = Cow::from(path.trim_start_matches('/'));

        let mut file = String::with_capacity(path.len() + 100);
        file.push_str(&self.config.pages_dir);
        file.push('/');

        if path.is_empty() {
            file.push_str(&self.config.index_file);
        } else {
            file.push_str(path.as_ref());

            file.push('/');
            if false == self.templates().any(|item| item.starts_with(&file)) {
                file.truncate(file.len() - 1);
            }

            file.push_str(&self.config.index_file);
            if false == self.templates().any(|item| item.starts_with(&file)) {
                file.truncate(file.len() - self.config.index_file.len());
            }

            if file.ends_with('/') {
                file.push_str("index.html");
            }

            if false == file.ends_with(".html") {
                file.push_str(".html");
            }
        }

        let context = req.html_render_context().await.unwrap();

        match self.render(file.as_str(), &context) {
            Ok(s) => Ok(HttpResponse::Ok().content_type(ContentType::html()).body(s)),
            Err(e) => {
                match e.kind {
                    tera::ErrorKind::TemplateNotFound(..) => Err(ErrorNotFound(e))?,
                    _ => {
                        log::error!("{e}: {:?}", e.source());
                        Err(ErrorInternalServerError(e))?
                    }
                }
            },
        }
    }

    #[inline]
    pub fn templates(&self) -> impl Iterator<Item = &str> {
        self.tera.get_template_names()
    }

    pub fn fetch_title(html: &str) -> Option<&str> {
        let open_pos = html.find("<title>");
        let close_pos = html.find("</title>");

        if open_pos.is_none() || close_pos.is_none() {
            return None;
        }

        html.get(open_pos.unwrap() + "<title>".len()..close_pos.unwrap())
            .map(|s| s.trim())
            .filter(|s| s.is_empty() == false)
    }
}

impl Debug for HtmlRender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlRender")
            .field("tera", &self.tera.get_template_names().collect::<Vec<_>>())
            .finish()
    }
}

impl Deref for HtmlRender {
    type Target = Tera;

    fn deref(&self) -> &Self::Target {
        &self.tera
    }
}

impl FromRequest for HtmlRender {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Box::pin(future::ok(
            req.app_data::<Arc<Self>>()
                .ok_or("HtmlRender does not exist in request.")
                .unwrap()
                .as_ref()
                .clone()
        ))
    }
}
