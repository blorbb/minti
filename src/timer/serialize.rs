use az::SaturatingAs;

use leptos::{SignalGetUntracked, SignalSetUntracked};
use serde::{Deserialize, Serialize};
use time::ext::NumericalDuration;

use crate::{contexts::TimerList, time::timestamp};

use super::{MultiTimer, RawMultiTimer};

/// A short, JSON representation of a timer.
#[derive(Debug, Serialize, Deserialize)]
struct TimerJson {
    /// The total duration (ms) of the timer.
    /// Defined if the timer has started.
    duration: Option<u64>,
    /// The unix timestamp (ms) of when the timer started.
    /// Defined if the timer has started.
    start: Option<i64>,
    /// The unix timestamp (ms) of when the timer was last paused.
    /// Defined if the timer is currently paused.
    last_pause: Option<i64>,
    /// The total duration (ms) that the timer has been paused for, excluding
    /// the current pause (if timer is paused).
    acc_pause_duration: u64,
    /// The string that was inputted for this timer.
    duration_input: String,
    /// The title given to the timer.
    title: String,
    /// Number of timers started.
    consumed: usize,
}

impl From<&RawMultiTimer> for TimerJson {
    fn from(value: &RawMultiTimer) -> Self {
        Self {
            duration: value
                .current_total_duration
                .get_untracked()
                .map(|d| d.whole_milliseconds().saturating_as::<u64>()),
            start: value
                .start_time
                .get_untracked()
                .map(timestamp::to_unix_millis),
            last_pause: value
                .last_pause_time
                .get_untracked()
                .map(timestamp::to_unix_millis),
            acc_pause_duration: value
                .acc_paused_duration
                .whole_milliseconds()
                .saturating_as::<u64>(),
            duration_input: value.input.get_untracked(),
            title: value.title.get_untracked(),
            consumed: value.consumed,
        }
    }
}

/// Transforms a `TimerList` into a JSON string.
pub fn stringify_timers(timers: TimerList) -> String {
    let timers: Vec<TimerJson> = timers
        .into_iter()
        .map(|timer| timer.0.with_value(|t| TimerJson::from(t)))
        .collect();
    serde_json::to_string(&timers).expect("Failed to convert timers to JSON")
}

/// Creates timers from the given JSON string.
///
/// Also sets the timers to the correct state.
///
/// The JSON string should be a list of timers, created by `stringify_timers`.
/// If any of the timers are invalid, they will be ignored.
///
/// Returns `None` if `json` could not be parsed.
pub fn parse_timer_json(json: &str) -> Option<TimerList> {
    let timers: Vec<TimerJson> = serde_json::from_str(json).ok()?;
    let timers: Vec<MultiTimer> = timers
        .into_iter()
        .filter_map(|unparsed| {
            let mut timer = RawMultiTimer::new();
            timer.input.set_untracked(unparsed.duration_input);
            timer.title.set_untracked(unparsed.title);

            if unparsed.consumed != 0 {
                timer.start().ok()?;
                // from 1 because start already advances the iterator
                (1..unparsed.consumed).for_each(|_| _ = timer.next());

                if unparsed.consumed != timer.consumed {
                    log::warn!(
                        "stored {} consumed, only managed to consume {}",
                        unparsed.consumed,
                        timer.consumed,
                    );
                    return None;
                }
            }

            // timer control methods (start, pause) set their respective properties to now.
            // must override the times after calling these methods.
            if let Some(duration) = unparsed.duration {
                timer
                    .set_current_total_duration
                    .set_untracked(Some(duration.saturating_as::<i64>().milliseconds()));
            }

            if let Some(start_time) = unparsed.start {
                timer.start_time.set_untracked(Some(timestamp::from_unix_millis(start_time)));
            };

            if let Some(last_pause_time) = unparsed.last_pause {
                // timer must also be started for it to be paused
                if !timer.started.get_untracked() {
                    return None;
                }

                timer.pause();
                timer.last_pause_time.set_untracked(Some(timestamp::from_unix_millis(last_pause_time)));
            }

            timer.acc_paused_duration = unparsed
                .acc_pause_duration
                .saturating_as::<i64>()
                .milliseconds();

            Some(MultiTimer(leptos::StoredValue::new(timer)))
        })
        .collect();

    Some(TimerList::from_timers(timers))
}
