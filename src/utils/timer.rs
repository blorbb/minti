pub mod serialize;

use leptos::*;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::time::relative;

/// A list of timers.
///
/// There will always be at least one timer. A new one is pushed
/// if the vector is empty.
///
/// `cx` should be the largest possible context (App).
#[derive(Debug, Clone)]
pub struct TimerList {
    vec: Vec<Timer>,
    cx: Scope,
}

impl TimerList {
    pub fn new(cx: Scope) -> Self {
        Self {
            vec: vec![Timer::new(cx)],
            cx,
        }
    }

    /// Gets the timer with a specific id.
    pub fn timer_with_id(&self, id: Uuid) -> Option<&Timer> {
        self.vec.iter().find(|t| t.id() == id)
    }

    pub fn from_timers(cx: Scope, timers: Vec<Timer>) -> Self {
        if timers.is_empty() {
            Self::new(cx)
        } else {
            Self { vec: timers, cx }
        }
    }

    pub fn push_new(&mut self) {
        self.vec.push(Timer::new(self.cx));
    }

    pub fn remove_index(&mut self, index: usize) -> Timer {
        let removed_timer = self.vec.remove(index);
        if self.is_empty() {
            self.push_new();
        };
        removed_timer
    }

    /// Removes the timer with the specified id.
    ///
    /// # Panics
    /// Panics if no timer with the given id is found.
    pub fn remove_id(&mut self, id: Uuid) -> Timer {
        let index = self
            .vec
            .iter()
            .position(|t| t.id() == id)
            .expect("Could not find timer with specified id.");
        self.remove_index(index)
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        self.push_new();
    }

    pub const fn as_vec(&self) -> &Vec<Timer> {
        &self.vec
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns whether this timer list is unchanged from initialisation.
    ///
    /// Checks that there is 1 timer with no input.
    pub fn is_initial(&self) -> bool {
        self.len() == 1 && self.as_vec()[0].input.get_untracked().is_empty()
    }
}

impl IntoIterator for TimerList {
    type Item = Timer;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

/// A timer that counts down.
///
/// Most of the inner components are reactive: subscribe to these properties
/// for reactivity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Timer {
    /// The total duration of the timer.
    ///
    /// Updates when the timer is started or reset.
    ///
    /// Is [`None`] if the timer hasn't started.
    pub duration: ReadSignal<Option<Duration>>,
    set_duration: WriteSignal<Option<Duration>>,
    /// Whether the timer has been started. If this is `true`, `duration`
    /// should also be `Some`.
    pub started: Memo<bool>,
    /// Whether the timer is counting down - started and not paused.
    pub running: Memo<bool>,
    /// Whether the timer is paused.
    pub paused: Memo<bool>,
    /// Whether the timer has reached 0.
    ///
    /// Updates after `update_time_remaining` is called.
    pub finished: Memo<bool>,
    /// A signal that stores the duration remaining.
    ///
    /// Updates when `update_time_remaining` is called. This should be used
    /// as the main way of accessing the time remaining.
    ///
    /// Is [`None`] if the timer hasn't started.
    pub time_remaining: ReadSignal<Option<Duration>>,
    set_time_remaining: WriteSignal<Option<Duration>>,
    /// A signal that stores time at which the timer will finish.
    ///
    /// Updates when `update_end_time` is called. This should be used
    /// as the main way of accessing the end time.
    ///
    /// Returns `None` if the timer is not started.
    /// If the timer is paused, it will return the end time as if the timer
    /// is resumed now.
    pub end_time: ReadSignal<Option<OffsetDateTime>>,
    set_end_time: WriteSignal<Option<OffsetDateTime>>,

    /// The string the user entered into this timer.
    ///
    /// Set using `Timer::set_input`
    pub input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    /// The title set for the timer (above each timer).
    ///
    /// Is an empty string if no title is set.
    pub title: ReadSignal<String>,
    set_title: WriteSignal<String>,

    // internal timekeeping stuff //
    /// The time at which the timer was started.
    start_time: RwSignal<Option<OffsetDateTime>>,
    /// The time of the last pause. Is `None` if the timer is not paused.
    last_pause_time: RwSignal<Option<OffsetDateTime>>,
    /// The accumulated amount of time that has been paused.
    ///
    /// Updates when the timer is resumed. This does not include time in the
    /// current pause, if the timer is paused.
    acc_paused_duration: RwSignal<Duration>,
    /// Notifies subscribers when any of the statuses (start, pause, finish)
    /// have changed. Get notified using `timer.state_change.track()`.
    pub state_change: Signal<()>,
    /// An id for the timer. Only to be used to distingush between different
    /// timers - this id is not stored in localstorage and will change if
    /// the timer is refetched from localstorage.
    id: Uuid,
}

impl Timer {
    /// Creates a new `Timer` instance.
    ///
    /// The timer is unstarted with zero duration.
    ///
    /// This should only be called in the largest scope (`App`) to avoid
    /// disposing the signals.
    pub fn new(cx: Scope) -> Self {
        let (duration, set_duration) = create_signal(cx, None::<Duration>);
        let start_time = create_rw_signal(cx, None);
        let last_pause_time = create_rw_signal(cx, None);
        let acc_paused_duration = create_rw_signal(cx, Duration::ZERO);

        let (time_remaining, set_time_remaining) = create_signal(cx, None::<Duration>);
        let (end_time, set_end_time) = create_signal(cx, None);
        let (input, set_input) = create_signal(cx, String::new());
        let (title, set_title) = create_signal(cx, String::new());

        let started = create_memo(cx, move |_| start_time().is_some());
        let paused = create_memo(cx, move |_| started() && last_pause_time().is_some());
        let running = create_memo(cx, move |_| started() && !paused());
        let finished = create_memo(cx, move |_| {
            time_remaining().is_some_and(|dur| !dur.is_positive())
        });

        // cannot be a memo: the return value does not change
        let state_change = Signal::derive(cx, move || {
            started.track();
            paused.track();
            running.track();
            finished.track();
            duration.track();
            leptos::log!("state changed");
        });

        Self {
            duration,
            set_duration,
            started,
            running,
            paused,
            finished,
            time_remaining,
            set_time_remaining,
            end_time,
            set_end_time,
            input,
            set_input,
            title,
            set_title,
            start_time,
            last_pause_time,
            acc_paused_duration,
            state_change,
            id: Uuid::new_v4(),
        }
    }

    /// Calculates the time elapsed as of now.
    ///
    /// Takes paused time into account.
    /// If the timer has not started, returns a zero duration.
    ///
    /// Does not update any signals.
    pub fn get_time_elapsed(&self) -> Duration {
        let Some(start_time) = (self.start_time).get_untracked() else {
            return Duration::ZERO;
        };

        let end_time = self
            .last_pause_time
            .get_untracked()
            .unwrap_or_else(relative::now);
        (end_time - start_time) - self.acc_paused_duration.get_untracked()
    }

    /// Calculates the time remaining in this timer as of now.
    ///
    /// Returns [`None`] if the timer hasn't started.
    /// If the timer is finished, a negative duration will be returned.
    pub fn get_time_remaining(&self) -> Option<Duration> {
        Some(self.duration.get_untracked()? - self.get_time_elapsed())
    }

    /// Updates the `time_remaining` signal.
    ///
    /// **Side effects:** Only way to update the `finished` signal.
    pub fn update_time_remaining(&self) {
        (self.set_time_remaining)(self.get_time_remaining());
    }

    /// Calculates the time at which the timer will finish.
    ///
    /// If the timer is paused, it will return the end time as if the timer
    /// is resumed now.
    ///
    /// Returns `None` if the timer hasn't started.
    pub fn get_end_time(&self) -> Option<OffsetDateTime> {
        if !self.started.get_untracked() {
            return None;
        }

        let duration_to_end = self.get_time_remaining()?;
        Some(relative::now() + duration_to_end)
    }

    /// Updates the `end_time` signal.
    pub fn update_end_time(&self) {
        (self.set_end_time)(self.get_end_time());
    }

    /// Resets the timer to as if a new one was created.
    pub fn reset(&self) {
        leptos::log!("resetting");
        self.start_time.set(None);
        self.last_pause_time.set(None);
        self.acc_paused_duration.set(Duration::ZERO);
        (self.set_duration)(None);
        self.update_time_remaining();
    }

    /// Starts the timer.
    pub fn start(&self, duration: Duration) {
        self.start_time.set(Some(relative::now()));
        (self.set_duration)(Some(duration));
    }

    /// Pauses the timer.
    ///
    /// Does not do anything if the timer is already paused.
    pub fn pause(&self) {
        if self.paused.get_untracked() {
            return;
        }

        self.last_pause_time.set(Some(relative::now()));
    }

    /// Resumes the timer.
    ///
    /// Does not do anything if the timer is not paused.
    pub fn resume(&self) {
        if !self.paused.get_untracked() {
            return;
        }

        self.acc_paused_duration.update(|v| {
            *v += relative::now() - self.last_pause_time.get_untracked().unwrap();
        });
        self.last_pause_time.set(None);
    }

    /// Adds a duration to the timer. Input a negative duration to subtract time.
    ///
    /// # Edge cases:
    /// - If the timer hasn't started, nothing will happen.
    /// - If duration is subtracted and results in the timer being finished, the
    ///   timer duration will be saturated to 0 and the timer will be unpaused.
    /// - If duration is subtracted while the timer has already finished,
    ///   nothing will happen.
    /// - If duration is added while the timer is already finished, the
    ///   overtime will be also added, so the timer jumps to having the
    ///   inputted duration remaining.
    pub fn add_duration(&self, duration: Duration) {
        if !self.started.get_untracked() {
            return;
        };

        // subtracting duration
        if duration.is_negative() {
            if self.finished.get_untracked() {
                return;
            } else if self.get_time_remaining().unwrap() <= -duration {
                // subtract will make timer finish: saturate at 0
                self.set_duration
                    .update(|d| *d = Some(d.unwrap() - self.get_time_remaining().unwrap()));
            } else {
                // nothing special: just subtract duration
                self.set_duration
                    .update(|d| *d = Some(d.unwrap() + duration));
            }
        } else if duration.is_positive() {
            if self.finished.get_untracked() {
                // timer is finished: add to make the remaining time `duration`.
                // subtract to add the negative duration
                self.set_duration.update(|d| {
                    *d = Some(d.unwrap() - self.get_time_remaining().unwrap() + duration);
                });
            } else {
                // nothing special: just add duration
                self.set_duration
                    .update(|d| *d = Some(d.unwrap() + duration));
            }
        };

        // push updates
        self.update_end_time();
        self.update_time_remaining();
    }

    /// Gets the id for the timer.
    ///
    /// Only to be used to distingush between different timers - this id is not
    /// stored in localstorage and will change if the timer is refetched from
    /// localstorage.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Sets the timer's input.
    pub fn set_input(&self, input: String) {
        (self.set_input)(input);
    }

    // Sets the timer's title.
    pub fn set_title(&self, title: String) {
        (self.set_title)(title);
    }
}
