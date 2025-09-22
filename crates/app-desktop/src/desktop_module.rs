use {
    crate::{DesktopConfig, DesktopConfigExt},
    app_base::prelude::*,
    core::marker::PhantomData
};

#[derive(Default)]
pub struct DesktopModule<C: DesktopConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for DesktopModule<C>
where
    C: DesktopConfigExt
{
    const COMMAND: &str = DesktopConfig::COMMAND;
    const DESCRIPTION: &str = "starts desktop application";

    type Config = C;

    fn run(&mut self, _app: &mut App<Self::Config>) -> Void {
        println!("OK");
        ok()
    }
}

/*
use {
    app_base::prelude::*,
    image::ImageReader,
    std::{env, io::Cursor, process::Command},
    tao::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Icon, WindowBuilder}
    },
    wry::{Result, WebViewBuilder}
};

enum UserEvent {
    LaunchUrl(String),
    LoadUrl(String)
}

fn main() -> Result<()> {
    let app_name = env!("APP_NAME").trim_matches(&['\'', '"']);
    let icon = include_bytes!(concat!(env!("PWD"), "/assets/desktop/icon.png"));
    let icon = ImageReader::new(Cursor::new(icon))
        .with_guessed_format()?
        .decode()
        .unwrap()
        .to_rgba8();
    let icon = Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap();

    let event_loop = EventLoop::new();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(app_name)
        .with_window_icon(Some(icon))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new()
        .with_url(env!("APP_DESKTOP_URL_START"))?
        .with_devtools(env::var("APP_DEBUG") == Ok("1".to_string()))
        .with_focused(true)
        .with_back_forward_navigation_gestures(true)
        .with_document_title_changed_handler(move |window, title| {
            window.set_title(&format!("{} - {title}", app_name))
        })
        .with_navigation_handler(|url| {
            if url.starts_with(env!("APP_DESKTOP_URL")) {
                return true;
            }

            if env::var("APP_ACCEPT_HOSTS")
                .unwrap()
                .split(',')
                .any(|host| {
                    url.split_once("://")
                        .filter(|(.., href)| href.starts_with(host.trim()))
                        .is_some()
                })
            {
                return true;
            }

            eprintln!("Forbidden url: {url}");

            false
        })
        .with_new_window_req_handler(move |url| {
            let file_name = match url.chars().position(|c| c == '?') {
                Some(pos) => Some(&url[0..pos]),
                None => Some(url.as_str())
            }
            .map(|file_name| {
                match file_name.rsplit_once('/') {
                    Some((.., file_name)) => file_name,
                    None => file_name
                }
            })
            .unwrap();

            if url.starts_with(env!("APP_DESKTOP_URL")) {
                if file_name.contains(".")
                    && false == file_name.to_ascii_lowercase().ends_with(".html")
                {
                    return proxy.send_event(UserEvent::LaunchUrl(url)).is_ok();
                }

                return proxy.send_event(UserEvent::LoadUrl(url)).is_ok();
            }

            if url.contains("://") {
                return proxy.send_event(UserEvent::LaunchUrl(url)).is_ok();
            }

            false
        })
        .with_ipc_handler(|_window, message| {
            log::trace!("Received IPC message: {}", &message);
        })
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => log::trace!("Started"),
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            },
            Event::UserEvent(UserEvent::LaunchUrl(url)) => {
                Command::new(
                    env::var("SHELL").expect("No environment variable 'SHELL'.")
                )
                .args(["-c", &format!("xdg-open \"{url}\" &")])
                .spawn()
                .map(|mut child| child.wait())
                .ok();
            },
            Event::UserEvent(UserEvent::LoadUrl(url)) => webview.load_url(&url),
            _ => ()
        }
    });
}
*/
