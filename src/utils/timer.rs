use chrono::{DateTime, Local};
use leptos::{
    create_rw_signal, create_signal, ReadSignal, RwSignal, Scope, SignalGetUntracked,
    SignalSet, SignalSetUntracked, SignalUpdateUntracked, WriteSignal,
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

// TODO use typestate?

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    // internal timekeeping stuff
    // using signals to mutate without needing `mut` and keeping `Copy`.
    // TODO change this to something else? RefCell does not impl `Copy`.
    // cannot use Instant as it doesn't work in wasm.
    start_time: RwSignal<Option<DateTime<Local>>>,
    /// The time of the last pause. Is `None` if the timer is not paused.
    last_pause_time: RwSignal<Option<DateTime<Local>>>,
    /// The total amount of time that has been paused.
    total_paused_duration: RwSignal<Duration>,
}

impl Timer {
    pub fn new(cx: Scope) -> Self {
        let (duration, set_duration) = create_signal(cx, Duration::ZERO);
        let (started, set_started) = create_signal(cx, false);
        let (running, set_running) = create_signal(cx, false);
        let (paused, set_paused) = create_signal(cx, false);
        let (finished, set_finished) = create_signal(cx, false);
        let (time_remaining, set_time_remaining) = create_signal(cx, Duration::ZERO);

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
            start_time: create_rw_signal(cx, None),
            last_pause_time: create_rw_signal(cx, None),
            total_paused_duration: create_rw_signal(cx, Duration::ZERO),
        }
    }

    pub fn reset_with_duration(&self, duration: Duration) {
        (self.set_started)(false);
        (self.set_running)(false);
        (self.set_paused)(false);
        (self.set_finished)(false);
        (self.set_duration)(duration);
        (self.set_time_remaining)(self.get_time_remaining());
        self.start_time.set_untracked(None);
    }

    fn get_time_elapsed(&self) -> Duration {
        let start_time = match (self.start_time).get_untracked() {
            Some(t) => t,
            None => return Duration::ZERO,
        };

        let end_time = self.last_pause_time.get_untracked().unwrap_or(Local::now());
        (end_time - start_time)
            .to_std()
            .expect("Start time to be before now")
            - self.total_paused_duration.get_untracked()
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
        self.start_time.set_untracked(Some(Local::now()));
        (self.set_started)(true);
        (self.set_running)(true);
    }

    pub fn pause(&self) {
        if self.paused.get_untracked() {
            return;
        }

        self.last_pause_time.set(Some(Local::now()));
        (self.set_paused)(true);
        (self.set_running)(false);
    }

    pub fn resume(&self) {
        if self.running.get_untracked() {
            return;
        }

        self.total_paused_duration.update_untracked(|v| {
            *v += (Local::now() - self.last_pause_time.get_untracked().unwrap())
                .to_std()
                .unwrap()
        });
        self.last_pause_time.set_untracked(None);
        (self.set_paused)(false);
        (self.set_running)(true);
    }
}
