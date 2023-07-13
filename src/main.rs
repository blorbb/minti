use leptos::*;
use minti_ui::app::App;

fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();

    mount_to_body(|cx| {
        view! { cx,
            <App/>
        }
    });
}
