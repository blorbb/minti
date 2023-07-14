use leptos::*;
use time::Duration;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;

use crate::utils::{reactive, timer::Timer};

#[expect(clippy::large_types_passed_by_value)]
#[component]
pub fn ProgressBar(cx: Scope, timer: Timer) -> impl IntoView {
    let elapsed = create_memo(cx, move |_| {
        timer.started.track();
        // timer.finished.track();
        timer.get_time_elapsed()
    });

    let progress_element = create_node_ref::<html::Div>(cx);

    create_effect(cx, move |_| {
        if !(timer.finished)() && let Some(progress_element) = progress_element() {
            reset_animation(progress_element.dyn_ref::<HtmlDivElement>().unwrap());
        }
    });

    view! { cx,
        <div
            class="com-progress-bar"
            role="progressbar"
            data-started=reactive::as_attr(timer.started)
            data-paused=reactive::as_attr(timer.paused)
            data-finished=reactive::as_attr(timer.finished)
        >
            <div
                class="progress-value"
                ref=progress_element
                style:animation-duration=move || {
                    format!("{:.3}s", (timer.duration) ().unwrap_or(Duration::MAX).as_seconds_f64())
                }
                style:animation-delay=move || { format!("{:.3}s", - elapsed().as_seconds_f64()) }
            ></div>

        // timer.finished.track();

        </div>
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
