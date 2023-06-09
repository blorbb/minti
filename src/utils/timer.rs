use chrono::{DateTime, Local};
use leptos::{create_rw_signal, RwSignal, Scope, SignalSet};
use uuid::Uuid;
use std::time::Duration;

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
    pub fn timer_with_id(&self, id: Uuid) -> &Timer {
        &self.0.iter().find(|t| t.id == id).unwrap().timer
    }

    /// Gets the timer with a specific id.
    ///
    /// # Panics
    /// Panics if the id cannot be found.
    pub fn timer_with_id_mut(&mut self, id: Uuid) -> &mut Timer {
        &mut self.0.iter_mut().find(|t| t.id == id).unwrap().timer
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
    id: Uuid,
    timer: Timer,
}

impl UniqueTimer {
    pub fn new(cx: Scope) -> Self {
        Self {
            id: Uuid::new_v4(),
            timer: Timer::new(cx)
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn timer(&self) -> Timer {
        self.timer
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timer {
    pub duration: Duration,
    start_time: DateTime<Local>,
    pub started: RwSignal<bool>,
    pub running: RwSignal<bool>,
    pub paused: RwSignal<bool>,
    /// Whether the timer has reached 0. Updates after `time_remaining()` is called.
    pub finished: RwSignal<bool>,
}

impl Timer {
    pub fn new(cx: Scope) -> Self {
        Self {
            duration: Duration::ZERO,
            start_time: Local::now(),
            started: create_rw_signal(cx, false),
            running: create_rw_signal(cx, false),
            paused: create_rw_signal(cx, false),
            finished: create_rw_signal(cx, false),
        }
    }

    pub fn reset_with_duration(&mut self, duration: Duration) -> &mut Self {
        self.started.set(false);
        self.running.set(false);
        self.paused.set(false);
        self.finished.set(false);
        self.start_time = Local::now();
        self.duration = duration;
        self
    }

    pub fn time_elapsed(&self) -> Duration {
        let start_time = self.start_time.timestamp_millis();
        let current_time = Local::now().timestamp_millis();

        // start time should be before current time
        Duration::from_millis((current_time - start_time) as u64)
    }

    /// Returns the time remaining in this timer.
    ///
    /// If the timer has finished, returns a zero duration.
    ///
    /// **Side effects:** Updates the `self.finished` property.
    pub fn time_remaining(&self) -> Duration {
        let time_remaining = self.duration.saturating_sub(self.time_elapsed());
        self.finished.set(time_remaining.is_zero());
        time_remaining
    }

    pub fn start(&mut self) {
        self.start_time = Local::now();
        self.started.set(true);
    }
}
