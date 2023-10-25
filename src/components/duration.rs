use time::Duration;

use leptos::*;
use leptos_mview::mview;

use crate::utils::time::units;

/// Displays a duration up to second accuracy.
///
/// Supports negative durations as well.
#[component]
pub fn DurationDisplay(#[prop(into)] duration: Signal<Duration>) -> impl IntoView {
    // need to exclude the millisecond unit, so the durations are slightly adjusted
    // to be displayed better.
    let ms = create_memo(move |_| duration().subsec_milliseconds().abs());
    // only show the negative sign once it hit -1 seconds (so that -0 is not shown).
    let show_negative = create_memo(move |_| duration() <= -Duration::SECOND);
    // Hits 0 when the duration finishes, not before.
    // i.e. 10.5 seconds shows as 11 instead of 10.
    // when duration becomes negative, no longer add 1 sec so that it just shows
    // whatever second it is. This avoids showing 0 for two seconds.
    let rounded_duration = create_memo(move |_| {
        if duration().is_negative() || ms() == 0 {
            duration().abs()
        } else {
            duration().abs() + Duration::SECOND
        }
    });
    let secs = create_memo(move |_| rounded_duration().whole_seconds() as u64 % units::SECS_IN_MIN);
    let mins =
        create_memo(move |_| rounded_duration().whole_minutes() as u64 % units::MINS_IN_HOUR);
    let hours = create_memo(move |_| rounded_duration().whole_hours() as u64 % units::HOURS_IN_DAY);
    let days = create_memo(move |_| rounded_duration().whole_days());

    mview! {
        span.com-duration {
            Show when={show_negative} fallback=[()] {
                span.negative { "-" }
            }
            Show when=[days() != 0] fallback=[()] {
                span.value { {days} }
                span.unit { "d" }
                " "
            }
            Show when=[hours() != 0 || days() != 0] fallback=[()] {
                span.value { {hours} }
                span.unit { "h" }
                " "
            }
            Show when=[mins() != 0 || hours() != 0 || days() != 0] fallback=[()] {
                span.value { {mins} }
                span.unit { "m" }
                " "
            }
            span.value { {secs} }
            span.unit { "s" }
        }
    }
}
