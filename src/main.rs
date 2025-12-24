mod curl_helper;
mod tinfoil_title_object;
mod utils;
mod window;

use curl::easy::Easy;
use curl_helper::BodyExt;
use gtk::{gio, glib, prelude::*};
use tinfoil_title_object::TinfoilRoot;
use window::Window;

const APP_ID: &str = "xyz.exefer.ns-emu-cheats-downloader";

fn main() -> glib::ExitCode {
    unsafe {
        std::env::set_var("GSK_RENDERER", "cairo");
    }

    gio::resources_register_include!("resources.gresource").expect("failed to register resources");

    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let window = Window::new(app);
        window.set_title(Some(concat!(
            env!("CARGO_PKG_NAME"),
            " - v",
            env!("CARGO_PKG_VERSION")
        )));

        window.present();

        glib::spawn_future_local(glib::clone!(
            #[weak]
            window,
            async move {
                let result = gio::spawn_blocking(move || {
                    let mut client = Easy::new();
                    client.url("https://tinfoil.io/Title/ApiJson/").unwrap();
                    client
                        .without_body()
                        .send_with_response::<TinfoilRoot>()
                        .unwrap()
                        .data
                })
                .await;
                if let Ok(titles) = result {
                    window.setup_titles(titles);
                }
            }
        ));
    });

    app.run()
}
