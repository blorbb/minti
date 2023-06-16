use std::time::Duration;

use leptos::*;

use crate::utils::duration::units;

#[component]
pub fn DurationDisplay(cx: Scope, #[prop(into)] duration: Signal<Duration>) -> impl IntoView {
    let ms = create_memo(cx, move |_| duration().as_millis() % 1000);
    let total_secs = create_memo(cx, move |_| {
        // Hits 0 when the duration finishes, not before.
        duration().as_secs() + if ms() == 0 { 0 } else { 1 }
    });
    let secs = create_memo(cx, move |_| total_secs() % units::SECS_IN_MIN);
    let total_mins = create_memo(cx, move |_| total_secs() / units::SECS_IN_MIN);
    let mins = create_memo(cx, move |_| total_mins() % units::MINS_IN_HOUR);
    let total_hours = create_memo(cx, move |_| total_mins() / units::MINS_IN_HOUR);
    let hours = create_memo(cx, move |_| total_hours() % units::HOURS_IN_DAY);
    let days = create_memo(cx, move |_| total_hours() / units::HOURS_IN_DAY);

    view! { cx,
        <span class="com-duration">
            <Show
                when=move || days() != 0
                fallback=|_| ()
            >
                <span class="value">{days}</span><span class="unit">"h"</span>" "
            </Show>
            <Show
                when=move || hours() != 0 || days() != 0
                fallback=|_| ()
            >
                <span class="value">{hours}</span><span class="unit">"h"</span>" "
            </Show>

            <Show
                when=move || mins() != 0 || hours() != 0 || days() != 0
                fallback=|_| ()
            >
                <span class="value">{mins}</span><span class="unit">"m"</span>" "
            </Show>

            <span class="value">{secs}</span><span class="unit">"s"</span>
        </span>
    }
}
