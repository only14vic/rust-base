use {
    crate::{DesktopConfig, DesktopConfigExt},
    app_base::prelude::*,
    core::marker::PhantomData,
    image::ImageReader,
    std::{io::Cursor, process::Command, rc::Rc},
    tao::{
        dpi::{LogicalPosition, LogicalSize},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoopBuilder},
        platform::run_return::EventLoopExtRunReturn,
        window::{Icon, WindowBuilder}
    },
    wry::{NewWindowResponse, Rect, WebViewBuilder}
};

#[derive(Debug)]
enum UserEvent {
    LaunchUrl(String),
    LoadUrl(String)
}

#[derive(Default)]
pub struct DesktopModule<C: DesktopConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for DesktopModule<C>
where
    C: DesktopConfigExt
{
    const COMMAND: &str = DesktopConfig::COMMAND;
    const DESCRIPTION: &str = "starts desktop application";

    type Config = C;

    fn run(&mut self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();
        let desktop_config = app.config().get::<DesktopConfig>().clone();
        let app_name = config.name.to_string();

        let icon = std::fs::read(&desktop_config.icon_path).unwrap();
        let icon = ImageReader::new(Cursor::new(icon))
            .with_guessed_format()?
            .decode()
            .unwrap()
            .to_rgba8();
        let icon = Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap();

        let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
        let proxy = event_loop.create_proxy();

        let window: Rc<_> = WindowBuilder::new()
            .with_title(&app_name)
            .with_window_icon(Some(icon))
            .build(&event_loop)?
            .into();

        let window_ref = window.clone();
        let window_size = window.inner_size().to_logical::<u32>(window.scale_factor());

        let webview_builder = WebViewBuilder::new()
            .with_url(&desktop_config.webview_start_url)
            .with_devtools(Env::is_debug())
            .with_focused(true)
            .with_back_forward_navigation_gestures(true)
            .with_bounds(Rect {
                position: LogicalPosition { x: 0, y: 0 }.into(),
                size: window_size.into()
            })
            .with_document_title_changed_handler(move |title| {
                let window = window_ref.clone();
                window.set_title(&format!("{app_name} - {title}"))
            })
            .with_ipc_handler(|message| {
                log::trace!("Received IPC message: {:#?}", &message);
            })
            .with_navigation_handler(move |url| {
                if url.starts_with(&desktop_config.webview_url) {
                    return true;
                }

                proxy.send_event(UserEvent::LaunchUrl(url)).unwrap();

                false
            });

        let desktop_config = app.config().get::<DesktopConfig>().clone();
        let proxy = event_loop.create_proxy();

        let webview_builder =
            webview_builder.with_new_window_req_handler(move |url, _| {
                if url.starts_with(&desktop_config.webview_url) {
                    proxy.send_event(UserEvent::LoadUrl(url)).unwrap();
                } else if url.contains("://") {
                    proxy.send_event(UserEvent::LaunchUrl(url)).unwrap();
                }

                NewWindowResponse::Deny
            });

        #[cfg(target_os = "linux")]
        let fixed = {
            use {gtk::prelude::*, tao::platform::unix::WindowExtUnix};
            let fixed = gtk::Fixed::new();
            let vbox = window.default_vbox().unwrap();
            vbox.pack_start(&fixed, true, true, 0);
            fixed.show_all();
            fixed
        };
        #[cfg(target_os = "linux")]
        let webview = {
            use wry::WebViewBuilderExtUnix;
            webview_builder.build_gtk(&fixed)?
        };
        #[cfg(not(target_os = "linux"))]
        let webview = webview_builder.build(&window)?;

        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    log::trace!("Started");
                },
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent {
                    event:
                        WindowEvent::Resized(..)
                        | WindowEvent::ScaleFactorChanged { .. }
                        | WindowEvent::Focused(..)
                        | WindowEvent::CursorEntered { .. }
                        | WindowEvent::CursorLeft { .. }
                        | WindowEvent::MouseInput { .. }
                        | WindowEvent::Touch(..),
                    ..
                } => {
                    let size =
                        window.inner_size().to_logical::<u32>(window.scale_factor());
                    webview
                        .set_bounds(Rect {
                            position: LogicalPosition { x: 0, y: 0 }.into(),
                            size: LogicalSize { width: size.width, height: size.height }
                                .into()
                        })
                        .unwrap();
                },
                Event::UserEvent(UserEvent::LaunchUrl(url)) => {
                    Command::new("sh")
                        .args(["-c", &format!("xdg-open \"{url}\" &")])
                        .spawn()
                        .map(|mut child| child.wait())
                        .unwrap()
                        .ok();
                },
                Event::UserEvent(UserEvent::LoadUrl(url)) => {
                    webview.load_url(&url).ok();
                },
                _ => {}
            }
        });

        ok()
    }
}
