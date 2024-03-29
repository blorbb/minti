use az::SaturatingAs;

use leptos::{SignalGetUntracked, SignalSetUntracked};
use serde::{Deserialize, Serialize};
use time::ext::NumericalDuration;

use crate::utils::{contexts::TimerList, time::timestamp};

use super::Timer;

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
}

impl From<Timer> for TimerJson {
    fn from(value: Timer) -> Self {
        Self {
            duration: value
                .duration
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
                .get_untracked()
                .whole_milliseconds()
                .saturating_as::<u64>(),
            duration_input: value.input.get_untracked(),
            title: value.title.get_untracked(),
        }
    }
}

/// Transforms a `TimerList` into a JSON string.
pub fn stringify_timers(timers: TimerList) -> String {
    let timers: Vec<TimerJson> = timers.into_iter().map(Into::into).collect();
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
    let timers: Vec<Timer> = timers
        .into_iter()
        .filter_map(|unparsed| {
            let timer = Timer::new();
            timer.set_input(unparsed.duration_input);
            timer.set_title(unparsed.title);

            // timer control methods (start, pause) set their respective properties to now.
            // must override the times after calling these methods.

            if let Some(start_time) = unparsed.start {
                timer.start(unparsed.duration?.saturating_as::<i64>().milliseconds());
                timer
                    .start_time
                    .set_untracked(Some(timestamp::from_unix_millis(start_time)));
            };

            if let Some(last_pause_time) = unparsed.last_pause {
                // timer must also be started for it to be paused
                if !timer.started.get_untracked() {
                    return None;
                }

                timer.pause();
                timer
                    .last_pause_time
                    .set_untracked(Some(timestamp::from_unix_millis(last_pause_time)));
            }

            timer.acc_paused_duration.set_untracked(
                unparsed
                    .acc_pause_duration
                    .saturating_as::<i64>()
                    .milliseconds(),
            );

            timer.update_time_remaining();
            timer.update_end_time();

            Some(timer)
        })
        .collect();

    Some(TimerList::from_timers(timers))
}
