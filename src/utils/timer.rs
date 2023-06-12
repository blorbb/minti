use chrono::{DateTime, Local};
use leptos::{
    create_rw_signal, create_signal, ReadSignal, RwSignal, Scope, SignalGetUntracked, WriteSignal,
};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TimerList(pub Vec<UniqueTimer>);

impl TimerList {
    pub fn new(cx: Scope, length: usize) -> Self {
        let vec = (0..length).map(|_| UniqueTimer::new(cx)).collect();
        Self(vec)
    }

    /// Gets the timer with a specific id.
    ///
    /// # Panics
    /// Panics if the id cannot be found.
    pub fn timer_with_id(&self, id: Uuid) -> Timer {
        (self.0.iter().find(|t| t.id == id).unwrap().timer)()
    }
}

impl IntoIterator for TimerList {
    type Item = UniqueTimer;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UniqueTimer {
    pub id: Uuid,
    pub timer: RwSignal<Timer>,
}

impl UniqueTimer {
    pub fn new(cx: Scope) -> Self {
        Self {
            id: Uuid::new_v4(),
            timer: create_rw_signal(cx, Timer::new(cx)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timer {
    pub duration: ReadSignal<Duration>,
    set_duration: WriteSignal<Duration>,
    start_time: ReadSignal<DateTime<Local>>,
    set_start_time: WriteSignal<DateTime<Local>>,
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
}

impl Timer {
    pub fn new(cx: Scope) -> Self {
        let (duration, set_duration) = create_signal(cx, Duration::ZERO);
        let (start_time, set_start_time) = create_signal(cx, Local::now());
        let (started, set_started) = create_signal(cx, false);
        let (running, set_running) = create_signal(cx, false);
        let (paused, set_paused) = create_signal(cx, false);
        let (finished, set_finished) = create_signal(cx, false);
        let (time_remaining, set_time_remaining) = create_signal(cx, Duration::ZERO);

        Self {
            duration,
            set_duration,
            start_time,
            set_start_time,
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
        }
    }

    pub fn reset_with_duration(&self, duration: Duration) {
        (self.set_started)(false);
        (self.set_running)(false);
        (self.set_paused)(false);
        (self.set_finished)(false);
        (self.set_start_time)(Local::now());
        (self.set_duration)(duration);
        (self.set_time_remaining)(self.get_time_remaining());
    }

    fn get_time_elapsed(&self) -> Duration {
        let start_time = self.start_time.get_untracked().timestamp_millis();
        let current_time = Local::now().timestamp_millis();

        // start time should be before current time
        Duration::from_millis((current_time - start_time) as u64)
    }

    /// Returns the time remaining in this timer.
    ///
    /// If the timer has finished, returns a zero duration.
    ///
    /// **Side effects:** Updates the `self.finished` property.
    fn get_time_remaining(&self) -> Duration {
        let time_remaining = self
            .duration
            .get_untracked()
            .saturating_sub(self.get_time_elapsed());
        if self.finished.get_untracked() != time_remaining.is_zero() {
            (self.set_finished)(time_remaining.is_zero());
        };
        time_remaining
    }

    pub fn update_time_remaining(&self) {
        (self.set_time_remaining)(self.get_time_remaining());
    }

    pub fn start(&self) {
        (self.set_start_time)(Local::now());
        (self.set_started)(true);
    }
}
