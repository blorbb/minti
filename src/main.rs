use leptos::*;
use minti_ui::app::App;

fn main() {
    mount_to_body(|cx| {
        view! { cx,
            <App/>
        }
    });
}
