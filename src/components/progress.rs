use leptos::*;
use leptos_mview::mview;
use time::Duration;
use web_sys::HtmlDivElement;

use crate::{reactive, timer::MultiTimer};

#[component]
pub fn ProgressBar(timer: MultiTimer, finished: Memo<bool>) -> impl IntoView {
    let elapsed = create_memo(move |_| {
        timer.started().track();
        finished.track();
        timer.time_elapsed()
    });

    let progress_element = NodeRef::<html::Div>::new();

    create_effect(move |_| {
        if !finished()
            && let Some(progress_element) = progress_element()
        {
            reset_animation(progress_element.as_ref());
        }
    });

    mview! {
        div.com-progress-bar
            role="progressbar"
            data-started={reactive::as_attr(timer.started())}
            data-paused={reactive::as_attr(timer.paused())}
            data-finished={reactive::as_attr(finished)}
        {
            div.progress-value
                ref={progress_element}
                style:animation-duration=f[
                    "{:.3}s",
                    timer.current_total_duration()().unwrap_or(Duration::MAX).as_seconds_f64()
                ]
                style:animation-delay=f["{:.3}s", -elapsed().as_seconds_f64()];
        }
    }
}

// https://css-tricks.com/restart-css-animation/#aa-update-another-javascript-method-to-restart-a-css-animation
fn reset_animation(element: &HtmlDivElement) {
    log::debug!("resetting element animation");
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
