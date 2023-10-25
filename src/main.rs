use leptos::*;
use leptos_mview::mview;
use minti_ui::app::App;

fn main() {
    console_log::init_with_level(log::Level::Trace).expect("logger should initialize");
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        mview! { App; }
    });
}
