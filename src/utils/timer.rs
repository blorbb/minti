pub mod serialize;

use chrono::{DateTime, Local};
use leptos::{
    create_effect, create_rw_signal, create_signal, ReadSignal, RwSignal, Scope,
    SignalGetUntracked, SignalSet, SignalSetUntracked, SignalUpdateUntracked, SignalWith,
    WriteSignal,
};
use std::time::Duration;
use uuid::Uuid;

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
        Self { vec: timers, cx }
    }

    pub fn push_new(&mut self) {
        self.vec.push(Timer::new(self.cx));
    }

    pub fn remove(&mut self, index: usize) -> Timer {
        let removed_timer = self.vec.remove(index);
        if self.is_empty() {
            self.push_new();
        };
        removed_timer
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

// TODO use typestate?

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Timer {
    pub duration: ReadSignal<Duration>,
    set_duration: WriteSignal<Duration>,
    pub started: ReadSignal<bool>,
    set_started: WriteSignal<bool>,
    pub running: ReadSignal<bool>,
    set_running: WriteSignal<bool>,
    pub paused: ReadSignal<bool>,
    set_paused: WriteSignal<bool>,
    /// Whether the timer has reached 0. Updates after `time_remaining()` is called.
    pub finished: ReadSignal<bool>,
    set_finished: WriteSignal<bool>,
    pub time_remaining: ReadSignal<Duration>,
    set_time_remaining: WriteSignal<Duration>,
    pub end_time: ReadSignal<Option<DateTime<Local>>>,
    set_end_time: WriteSignal<Option<DateTime<Local>>>,
    pub input: ReadSignal<String>,
    pub set_input: WriteSignal<String>,
    // internal timekeeping stuff
    // using signals to mutate without needing `mut` and keeping `Copy`.
    // TODO change this to something else? RefCell does not impl `Copy`.
    // cannot use Instant as it doesn't work in wasm.
    start_time: RwSignal<Option<DateTime<Local>>>,
    /// The time of the last pause. Is `None` if the timer is not paused.
    last_pause_time: RwSignal<Option<DateTime<Local>>>,
    /// The total amount of time that has been paused.
    total_paused_duration: RwSignal<Duration>,
    /// Notifies subscribers when any of the statuses (start, pause, finish)
    /// have changed. Get notified using `timer.state_change.track()`.
    pub state_change: RwSignal<()>,
    id: Uuid,
}

impl Timer {
    pub fn new(cx: Scope) -> Self {
        let (duration, set_duration) = create_signal(cx, Duration::ZERO);
        let (started, set_started) = create_signal(cx, false);
        let (running, set_running) = create_signal(cx, false);
        let (paused, set_paused) = create_signal(cx, false);
        let (finished, set_finished) = create_signal(cx, false);
        let (time_remaining, set_time_remaining) = create_signal(cx, Duration::ZERO);
        let (input, set_input) = create_signal(cx, String::new());
        let (end_time, set_end_time) = create_signal(cx, None);

        let state_change = create_rw_signal(cx, ());
        create_effect(cx, move |_| {
            input.track();
            state_change.set(());
        });

        Self {
            duration,
            set_duration,
            started,
            set_started,
            running,
            set_running,
            paused,
            set_paused,
            finished,
            set_finished,
            time_remaining,
            set_time_remaining,
            end_time,
            set_end_time,
            input,
            set_input,
            start_time: create_rw_signal(cx, None),
            last_pause_time: create_rw_signal(cx, None),
            total_paused_duration: create_rw_signal(cx, Duration::ZERO),
            state_change: create_rw_signal(cx, ()),
            id: Uuid::new_v4(),
        }
    }

    pub fn notify_state_change(&self) {
        self.state_change.set(());
    }

    pub fn get_time_elapsed(&self) -> Duration {
        let Some(start_time) = (self.start_time).get_untracked() else { return Duration::ZERO };

        let end_time = self
            .last_pause_time
            .get_untracked()
            .unwrap_or_else(Local::now);
        (end_time - start_time)
            .clamp(chrono::Duration::zero(), chrono::Duration::max_value())
            .to_std()
            .expect("Clamped duration is non-negative")
            - self.total_paused_duration.get_untracked()
    }

    /// Returns the time remaining in this timer.
    ///
    /// If the timer has finished, returns a zero duration.
    ///
    /// **Side effects:** Updates the `self.finished` property.
    pub fn get_time_remaining(&self) -> Duration {
        let time_remaining = self
            .duration
            .get_untracked()
            .saturating_sub(self.get_time_elapsed());

        if self.finished.get_untracked() != time_remaining.is_zero() {
            (self.set_finished)(time_remaining.is_zero());
            self.notify_state_change();
        };

        time_remaining
    }

    pub fn update_time_remaining(&self) {
        (self.set_time_remaining)(self.get_time_remaining());
    }

    #[expect(clippy::missing_panics_doc, reason = "it won't (hopefully)")]
    pub fn get_end_time(&self) -> Option<DateTime<Local>> {
        if !self.started.get_untracked() {
            return None;
        }
        let now = Local::now();
        let duration_to_end = chrono::Duration::from_std(self.get_time_remaining()).unwrap();
        Some(now + duration_to_end)
    }

    pub fn update_end_time(&self) {
        (self.set_end_time)(self.get_end_time());
    }

    pub fn reset_with_duration(&self, duration: Duration) {
        (self.set_started)(false);
        (self.set_running)(false);
        (self.set_paused)(false);
        (self.set_finished)(false);
        (self.set_duration)(duration);
        (self.set_time_remaining)(self.get_time_remaining());
        self.start_time.set_untracked(None);
        self.last_pause_time.set_untracked(None);
        self.total_paused_duration.set_untracked(Duration::ZERO);
        self.notify_state_change();
    }

    pub fn reset(&self) {
        self.reset_with_duration(Duration::ZERO);
    }

    pub fn start(&self) {
        self.start_time.set_untracked(Some(Local::now()));
        (self.set_started)(true);
        (self.set_running)(true);
        self.notify_state_change();
    }

    pub fn pause(&self) {
        if self.paused.get_untracked() {
            return;
        }

        self.last_pause_time.set(Some(Local::now()));
        (self.set_paused)(true);
        (self.set_running)(false);
        self.notify_state_change();
    }

    #[expect(
        clippy::missing_panics_doc,
        reason = "last_pause_time should be defined if timer is not running"
    )]
    pub fn resume(&self) {
        if self.running.get_untracked() {
            return;
        }

        self.total_paused_duration.update_untracked(|v| {
            *v += (Local::now() - self.last_pause_time.get_untracked().unwrap())
                .to_std()
                .unwrap();
        });
        self.last_pause_time.set_untracked(None);
        (self.set_paused)(false);
        (self.set_running)(true);
        self.notify_state_change();
    }

    pub const fn id(&self) -> Uuid {
        self.id
    }
}
