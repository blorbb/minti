use leptos::*;
use minti_ui::app::App;

fn main() {
    console_log::init_with_level(log::Level::Trace).expect("logger should initialize");
    console_error_panic_hook::set_once();

    mount_to_body(|cx| {
        view! { cx, <App/> }
    });
}
