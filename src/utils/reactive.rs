use std::time::Duration;

use leptos::{create_effect, leptos_dom::helpers::IntervalHandle, set_interval_with_handle, Scope};

/// Runs a callback and repeats it while `when` is true.
pub fn repeat_while(
    cx: Scope,
    when: impl Fn() -> bool + 'static,
    callback: impl Fn() + Clone + 'static,
    duration: Duration,
) {
    // needs double Option as the outer one is None on first run,
    // but needs to be None if when() is false.
    #[expect(clippy::option_option, reason = "required")]
    create_effect(cx, move |prev_handle: Option<Option<IntervalHandle>>| {
        // cancel the previous handle if it exists
        if let Some(prev_handle) = prev_handle.flatten() {
            prev_handle.clear();
        };

        // return handle so that next call can access it
        if when() {
            callback();
            Some(
                set_interval_with_handle(callback.clone(), duration)
                    .expect("Could not create interval"),
            )
        } else {
            None
        }
    });
}
