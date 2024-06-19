pub mod serialize;

use leptos::*;
use std::{cell::RefCell, sync::Arc};
use std::rc::Rc;
use std::time::Duration as StdDuration;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    commands,
    interpreter::{self, InputIter},
    reactive,
};

use super::time::relative;

macro_rules! prop {
    ($prop:ident: $ty:ty) => {
        pub fn $prop(&self) -> $ty {
            self.0.with_value(|t| t.$prop)
        }
    };
}

macro_rules! priv_prop {
    ($prop:ident: $ty:ty) => {
        fn $prop(&self) -> $ty {
            self.0.with_value(|t| t.$prop)
        }
    };
}

macro_rules! method {
    ($method:ident( $( $arg:ident: $ty:ty ),* ) $(-> $ret:ty)?) => {
        pub fn $method(&self $(, $arg: $ty)* ) $(-> $ret)? {
            self.0.with_value(|t| t.$method( $($arg),* ))
        }
    };
}

#[derive(Clone, Copy)]
pub struct MultiTimer(StoredValue<RawMultiTimer>);

impl MultiTimer {
    prop!(current: Timer);
    prop!(input: ReadSignal<String>);
    prop!(title: ReadSignal<String>);
    prop!(consumed: usize);
    prop!(id: Uuid);
    method!(set_input(input: String));
    method!(set_title(title: String));

    pub fn new() -> Self {
        Self(StoredValue::new(RawMultiTimer::new()))
    }

    pub fn restart(&self, iter: InputIter) {
        batch(|| {
            self.reset();
            self.start(iter);
        });
    }

    pub fn reset(&self) {
        self.current().reset();
        self.0.update_value(|t| t.consumed = 0);
        self.0
            .with_value(|t| *t.iter.borrow_mut() = InputIter::empty());
    }

    /// This must be called with consumed set to 0
    fn start(&self, iter: InputIter) {
        self.0.with_value(|t| *t.iter.borrow_mut() = iter);
        self.next();
    }

    pub fn next(&self) {
        batch(|| {
            let Some(next) = self.0.with_value(|t| t.iter.borrow_mut().next()) else {
                return;
            };
            self.0.update_value(|timer| {
                timer.consumed += 1;
                log::info!("consumed, timer.consumed = {}", timer.consumed);
            });

            self.current().restart(
                interpreter::interpret_single(&next).expect("input has already been validated"),
            );
        })
    }

    pub fn peek(&self) -> Option<Arc<str>> {
        self.0.with_value(|t| t.iter.borrow_mut().peek())
    }
}

pub struct RawMultiTimer {
    pub current: Timer,
    iter: RefCell<InputIter>,
    pub input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    pub title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    consumed: usize,
    id: Uuid,
}

impl RawMultiTimer {
    pub fn new() -> Self {
        let (input, set_input) = create_signal(String::new());
        let (title, set_title) = create_signal(String::new());
        Self {
            current: Timer::new(),
            iter: RefCell::new(InputIter::empty()),
            input,
            set_input,
            title,
            set_title,
            consumed: 0,
            id: Uuid::new_v4(),
        }
    }

    /// Sets the timer's input.
    pub fn set_input(&self, input: String) {
        log::trace!("setting input to {:?}", input);
        (self.set_input)(input);
    }

    // Sets the timer's title.
    pub fn set_title(&self, title: String) {
        log::trace!("setting title to {:?}", title);
        (self.set_title)(title);
    }

    /// Gets the id for the timer.
    ///
    /// Only to be used to distingush between different timers - this id is not
    /// stored in localstorage and will change if the timer is refetched from
    /// localstorage.
    pub const fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Timer(StoredValue<RawTimer>);

impl Timer {
    prop!(duration: ReadSignal<Option<Duration>>);
    prop!(started: Memo<bool>);
    prop!(running: Memo<bool>);
    prop!(paused: Memo<bool>);
    prop!(finished: Memo<bool>);
    prop!(time_remaining: ReadSignal<Option<Duration>>);
    prop!(end_time: ReadSignal<Option<OffsetDateTime>>);
    // priv_prop!(set_duration: WriteSignal<Option<Duration>>);
    // priv_prop!(set_time_remaining: WriteSignal<Option<Duration>>);
    // priv_prop!(set_end_time: WriteSignal<Option<OffsetDateTime>>);
    priv_prop!(start_time: RwSignal<Option<OffsetDateTime>>);
    priv_prop!(last_pause_time: RwSignal<Option<OffsetDateTime>>);
    priv_prop!(acc_paused_duration: RwSignal<Duration>);

    method!(get_time_elapsed() -> Duration);
    method!(get_time_remaining() -> Option<Duration>);
    method!(update_time_remaining());
    method!(get_end_time() -> Option<OffsetDateTime>);
    method!(update_end_time());
    method!(restart(duration: Duration));
    method!(reset());
    method!(start(duration: Duration));
    method!(pause());
    method!(resume());
    method!(add_duration(duration: Duration));

    pub fn set_after_finish(&self, closure: impl Fn() + 'static) {
        self.0.update_value(|t| t.set_after_finish(closure));
    }

    pub fn new() -> Self {
        let timer = Self(StoredValue::new(RawTimer::new()));

        // update the time remaining when the timer is running
        reactive::repeat_while(
            timer.running(),
            move || timer.update_time_remaining(),
            StdDuration::from_millis(200),
        );

        // update the end time when the timer is paused (started and not running)
        reactive::repeat_while(
            timer.paused(),
            move || timer.update_end_time(),
            StdDuration::SECOND,
        );
        // also need to update when the timer resets,
        // so that the end time component is removed
        create_effect(move |_| {
            timer.started().track();
            timer.update_end_time();
        });

        // request for user attention when the timer finishes //

        create_effect(move |_| {
            // also check that it is close to finish so that already expired timers
            // retrieved from localstorage don't alert
            if (timer.finished())()
                && timer
                    .get_time_remaining()
                    .expect("timer is finished => should have started")
                    .abs()
                    < Duration::SECOND
            {
                spawn_local(commands::alert_window());
            };
        });

        timer
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// A timer that counts down.
///
/// Most of the inner components are reactive: subscribe to these properties
/// for reactivity.
pub struct RawTimer {
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
    /// An optional closure to run immediately after the timer finishes.
    ///
    /// The closure will run after signals are updated.
    after_finish: Option<Rc<dyn Fn()>>,
}

impl RawTimer {
    /// Creates a new `Timer` instance.
    ///
    /// The timer is unstarted with zero duration.
    ///
    /// This should only be called in the largest scope (`App`) to avoid
    /// disposing the signals.
    pub fn new() -> Self {
        log::info!("creating new timer");

        let (duration, set_duration) = create_signal(None::<Duration>);
        let start_time = create_rw_signal(None);
        let last_pause_time = create_rw_signal(None);
        let acc_paused_duration = create_rw_signal(Duration::ZERO);

        let (time_remaining, set_time_remaining) = create_signal(None::<Duration>);
        let (end_time, set_end_time) = create_signal(None);

        let started = create_memo(move |_| start_time().is_some());
        let paused = create_memo(move |_| started() && last_pause_time().is_some());
        let running = create_memo(move |_| started() && !paused());
        let finished = create_memo(move |_| time_remaining().is_some_and(|dur| !dur.is_positive()));

        // cannot be a memo: the return value does not change
        let state_change = Signal::derive(move || {
            started.track();
            paused.track();
            running.track();
            finished.track();
            duration.track();
            log::trace!("state changed");
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
            start_time,
            last_pause_time,
            acc_paused_duration,
            state_change,
            after_finish: None,
        }
    }

    /// Calculates the time elapsed as of now.
    ///
    /// Takes paused time into account.
    /// If the timer has not started, returns a zero duration.
    ///
    /// Does not update any signals.
    pub fn get_time_elapsed(&self) -> Duration {
        // log::trace!("getting time elapsed");

        let Some(start_time) = (self.start_time).get_untracked() else {
            log::info!("timer has not started: returning 0 duration");
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
        // log::trace!("getting time remaining");
        Some(self.duration.get_untracked()? - self.get_time_elapsed())
    }

    /// Updates the `time_remaining` signal.
    ///
    /// **Side effects:** Only way to update the `finished` signal.
    pub fn update_time_remaining(&self) {
        // log::trace!("updating time remaining");
        let time_remaining = self.get_time_remaining();
        if time_remaining.is_some_and(|dur| !dur.is_positive())
            && !self.finished.get_untracked()
            && let Some(after_finish) = &self.after_finish
        {
            (self.set_time_remaining)(self.get_time_remaining());
            after_finish();
            (self.set_time_remaining)(self.get_time_remaining());
        } else {
            (self.set_time_remaining)(time_remaining);
        }
    }

    pub fn set_after_finish(&mut self, closure: impl Fn() + 'static) {
        self.after_finish = Some(Rc::new(closure));
    }

    /// Calculates the time at which the timer will finish.
    ///
    /// If the timer is paused, it will return the end time as if the timer
    /// is resumed now.
    ///
    /// Returns `None` if the timer hasn't started.
    pub fn get_end_time(&self) -> Option<OffsetDateTime> {
        // log::trace!("getting end time");
        if !self.started.get_untracked() {
            log::trace!("timer hasn't started, returning None");
            return None;
        }

        let duration_to_end = self.get_time_remaining()?;
        Some(relative::now() + duration_to_end)
    }

    /// Updates the `end_time` signal.
    pub fn update_end_time(&self) {
        // log::trace!("updating end time");
        (self.set_end_time)(self.get_end_time());
    }

    /// Batches `Timer::reset` and `Timer::start`.
    pub fn restart(&self, duration: Duration) {
        log::debug!("restarting timer");
        batch(|| {
            self.reset();
            // force update the running memo
            // so that the interval loop restarts. restart loop to avoid being
            // highly offset from the actual time (200ms)
            self.running.get_untracked();
            self.start(duration);
        });
    }

    /// Resets the timer to as if a new one was created, but keeping the
    /// initial input and title.
    pub fn reset(&self) {
        log::debug!("resetting timer");
        batch(|| {
            self.start_time.set(None);
            self.last_pause_time.set(None);
            self.acc_paused_duration.set(Duration::ZERO);
            (self.set_duration)(None);
            self.update_time_remaining();
        });
    }

    /// Starts the timer.
    pub fn start(&self, duration: Duration) {
        log::debug!("starting timer with duration {}", duration);
        batch(|| {
            self.start_time.set(Some(relative::now()));
            (self.set_duration)(Some(duration));
        });
    }

    /// Pauses the timer.
    ///
    /// Does not do anything if the timer is already paused.
    pub fn pause(&self) {
        log::debug!("paused timer");
        if self.paused.get_untracked() {
            log::info!("timer already paused, pause did nothing");
            return;
        }

        self.last_pause_time.set(Some(relative::now()));
    }

    /// Resumes the timer.
    ///
    /// Does not do anything if the timer is not paused.
    pub fn resume(&self) {
        log::debug!("resuming timer");
        if !self.paused.get_untracked() {
            log::info!("timer already paused, resume did nothing");
            return;
        }

        batch(|| {
            self.acc_paused_duration.update(|v| {
                *v += relative::now()
                    - self
                        .last_pause_time
                        .get_untracked()
                        .expect("timer should be paused");
            });
            self.last_pause_time.set(None);
        });
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
    ///   timer will be reset with the duration specified.
    pub fn add_duration(&self, duration: Duration) {
        log::debug!("adding duration {} to timer", duration);
        if !self.started.get_untracked() {
            log::warn!("timer hasn't started, not changing duration");
            return;
        };

        // NOTE: do not use `self.duration.update` if need to use
        // `self.get_time_remaining`. `update` borrows `self.duration` mutably
        // and  `get_time_remaining` tries to borrow `self.duration` immutably,
        // causing panic.
        // use `self.set_duration.set` instead.

        batch(|| {
            // subtracting duration
            if duration.is_negative() {
                if self.finished.get_untracked() {
                    log::warn!("timer is already finished: doing nothing");
                    return;
                } else if self
                    .get_time_remaining()
                    .expect("timer should have started")
                    <= -duration
                {
                    // subtract will make timer finish: saturate at 0
                    let new_duration =
                        self.duration.get_untracked().unwrap() - self.get_time_remaining().unwrap();
                    log::trace!(
                        "saturating duration to finish: subtracting {}",
                        new_duration
                    );
                    // unpause the timer to start overtime countdown
                    self.resume();

                    (self.set_duration)(Some(new_duration));
                } else {
                    log::trace!("subtracting duration");
                    // nothing special: just subtract duration
                    self.set_duration
                        .update(|d| *d = Some(d.unwrap() + duration));
                }
            } else if duration.is_positive() {
                if self.finished.get_untracked() {
                    log::debug!("restarting to {}", duration);
                    self.restart(duration);
                } else {
                    log::trace!("adding duration");
                    // nothing special: just add duration
                    self.set_duration
                        .update(|d| *d = Some(d.unwrap() + duration));
                }
            };

            // push updates
            self.update_end_time();
            self.update_time_remaining();
        });
    }
}
