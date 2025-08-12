use {
    super::CurrentUser,
    actix::spawn,
    actix_web::{HttpRequest, Responder, rt::time::sleep, web},
    actix_ws::{Closed, Message},
    app_base::prelude::*,
    futures::{FutureExt, StreamExt},
    std::time::{Duration, SystemTime}
};

pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    _current_user: CurrentUser
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    spawn(async move {
        let sess = &mut session;
        let mut closed_reason = None;
        let mut time = SystemTime::now();

        sess.text("Hello".to_json().unwrap().to_string()).await.ok();

        loop {
            let Some(msg) = stream.next().now_or_never() else {
                sleep(Duration::from_millis(100)).await;

                let now = SystemTime::now();
                if now.duration_since(time).unwrap().as_secs() >= 3 {
                    if sess.ping(b"").await.is_err() {
                        break;
                    }
                    time = now;
                }

                continue;
            };

            let Some(Ok(msg)) = msg else {
                break;
            };

            let res = match msg {
                Message::Continuation(msg) => sess.continuation(msg).await,
                Message::Ping(bytes) => sess.pong(&bytes).await,
                Message::Text(msg) => sess.text(msg).await,
                Message::Close(reason) => {
                    closed_reason = reason;
                    Err(Closed)
                },
                _ => Ok(())
            };

            if res.is_err() {
                break;
            }
        }

        session.close(closed_reason).await.ok()
    });

    Ok(response)
}
