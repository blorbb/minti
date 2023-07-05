use std::time::Duration;

use leptos::{
    create_effect, create_rw_signal, leptos_dom::helpers::IntervalHandle, on_cleanup,
    set_interval_with_handle, MaybeSignal, Scope, SignalSet,
};

/// Runs a callback and repeats it while `when` is true.
pub fn repeat_while(
    cx: Scope,
    when: impl Fn() -> bool + 'static,
    callback: impl Fn() + Clone + 'static,
    duration: Duration,
) {
    // while the effect will be stopped when `cx` is disposed, the interval
    // is not destroyed.
    // Extra signal to stop the interval when `cx` is disposed.
    let stop = create_rw_signal(cx, false);
    on_cleanup(cx, move || stop.set(true));

    // needs double Option as the outer one is None on first run,
    // but needs to be None if when() is false.
    #[expect(clippy::option_option, reason = "required")]
    create_effect(cx, move |prev_handle: Option<Option<IntervalHandle>>| {
        // cancel the previous handle if it exists
        if let Some(prev_handle) = prev_handle.flatten() {
            prev_handle.clear();
        };

        // return handle so that next call can access it
        if when() && !stop() {
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

/// Converts a bool signal into a string. For use in HTML attributes that
/// require a "true" or "false" value.
///
/// # Example
/// ```rust
/// # use leptos::*;
/// # use minti_ui::utils::reactive;
/// # #[component]
/// # pub fn Test(cx: Scope) -> impl IntoView {
/// let (started, set_started) = create_signal(cx, false);
/// view! { cx,
///     // becomes <div data-started="false"></div>
///     <div data-started=reactive::as_attr(started)></div>
/// }
/// # }
/// ```
pub fn as_attr(bool_signal: impl Into<MaybeSignal<bool>>) -> impl Fn() -> String {
    let bool_signal: MaybeSignal<bool> = bool_signal.into();
    move || bool_signal().to_string()
}
