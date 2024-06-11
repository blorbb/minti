use leptos::*;
use leptos_mview::mview;
use time::Duration;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;

use crate::utils::{reactive, timer::Timer};

#[component]
pub fn ProgressBar(timer: Timer) -> impl IntoView {
    let elapsed = create_memo(move |_| {
        timer.started().track();
        // timer.finished.track();
        timer.get_time_elapsed()
    });

    let progress_element = create_node_ref::<html::Div>();

    create_effect(move |_| {
        if !(timer.finished())() && let Some(progress_element) = progress_element() {
            reset_animation(progress_element.dyn_ref::<HtmlDivElement>().unwrap());
        }
    });

    mview! {
        div.com-progress-bar
            role="progressbar"
            data-started={reactive::as_attr(timer.started())}
            data-paused={reactive::as_attr(timer.paused())}
            data-finished={reactive::as_attr(timer.finished())}
        {
            div.progress-value
                ref={progress_element}
                style:animation-duration=[
                    format!("{:.3}s", (timer.duration())().unwrap_or(Duration::MAX).as_seconds_f64())
                ]
                style:animation-delay=[format!("{:.3}s", -elapsed().as_seconds_f64())];
        }
    }
}

// https://css-tricks.com/restart-css-animation/#aa-update-another-javascript-method-to-restart-a-css-animation
fn reset_animation(element: &HtmlDivElement) {
    let previous_animation = element
        .style()
        .get_property_value("animation-name")
        .unwrap();
    element
        .style()
        .set_property("animation-name", "none")
        .unwrap();
    // trigger reflow
    _ = element.offset_width();
    element
        .style()
        .set_property("animation-name", &previous_animation)
        .unwrap();
}
