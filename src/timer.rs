use leptos::*;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::interpreter::{self, interpret_multi, interpret_single, InputIter};
use crate::time::relative;

pub mod serialize;

macro_rules! prop {
    ($prop:ident: $ty:ty) => {
        pub fn $prop(&self) -> $ty {
            self.0.with_value(|t| t.$prop)
        }
    };
}

macro_rules! method {
    // immutable method
    ($method:ident( &self $(, $arg:ident: $ty:ty )* ) $(-> $ret:ty)?) => {
        pub fn $method(&self $(, $arg: $ty)* ) $(-> $ret)? {
            batch(|| {
                self.0.with_value(|t| t.$method( $($arg),* ))
            })
        }
    };

    // mutable method that doesn't return
    ($method:ident( &mut self $(, $arg:ident: $ty:ty )* )) => {
        pub fn $method(&self $(, $arg: $ty)* ) {
            batch(|| {
                self.0.update_value(|t| t.$method( $($arg),* ))
            })
        }
    };

    // mutable method that returns
    ($method:ident( &mut self $(, $arg:ident: $ty:ty )* ) -> $ret:ty) => {
        pub fn $method(&self $(, $arg: $ty)* ) -> $ret {
            batch(|| {
                let mut x = None;
                self.0.update_value(|t| x = Some(t.$method( $($arg),* )));
                x.expect("should have updated value")
            })
        }
    };
}

#[derive(Clone, Copy)]
pub struct MultiTimer(StoredValue<RawMultiTimer>);

impl MultiTimer {
    prop!(input: RwSignal<String>);
    prop!(title: RwSignal<String>);
    prop!(started: Memo<bool>);
    prop!(paused: Memo<bool>);
    prop!(running: Memo<bool>);
    prop!(current_total_duration: ReadSignal<Option<Duration>>);
    prop!(status_update: Trigger);

    method!(next(&mut self) -> Option<Duration>);
    method!(peek(&mut self) -> Option<Arc<str>>);
    method!(reset(&mut self));
    method!(start(&mut self) -> interpreter::Result<()>);
    method!(pause(&mut self));
    method!(resume(&mut self));
    method!(add_duration(&mut self, duration: Duration));
    method!(time_elapsed(&self) -> Duration);
    method!(id(&self) -> Uuid);

    pub fn new() -> Self {
        Self(StoredValue::new(RawMultiTimer::new()))
    }
}

struct RawMultiTimer {
    pub input: RwSignal<String>,
    pub title: RwSignal<String>,

    id: Uuid,

    // Internal stuff
    /// The time at which the timer was started.
    start_time: RwSignal<Option<OffsetDateTime>>,
    /// The time of the last pause. Is `None` if the timer is not paused.
    last_pause_time: RwSignal<Option<OffsetDateTime>>,
    /// The accumulated amount of time that has been paused.
    ///
    /// Updates when the timer is resumed. This does not include time in the
    /// current pause, if the timer is paused.
    acc_paused_duration: Duration,
    iter: InputIter,
    /// Number of timers started, i.e. number of times `next` has been called.
    consumed: usize,
    pub current_total_duration: ReadSignal<Option<Duration>>,
    set_current_total_duration: WriteSignal<Option<Duration>>,

    // Status signals
    pub started: Memo<bool>,
    pub paused: Memo<bool>,
    pub running: Memo<bool>,
    pub status_update: Trigger,
}

impl RawMultiTimer {
    pub fn new() -> Self {
        log::info!("creating new timer");

        let start_time = RwSignal::new(None);
        let last_pause_time = RwSignal::new(None);

        let started = Memo::new(move |_| start_time().is_some());
        let paused = Memo::new(move |_| started() && last_pause_time().is_some());
        let running = Memo::new(move |_| started() && !paused());
        let (current_total_duration, set_current_total_duration) = create_signal(None);

        let status_update = Trigger::new();
        Effect::new(move |_| {
            started.track();
            paused.track();
            current_total_duration.track();
            status_update.notify();
        });

        Self {
            input: RwSignal::new(String::new()),
            title: RwSignal::new(String::new()),
            id: Uuid::new_v4(),
            start_time,
            last_pause_time,
            acc_paused_duration: Duration::ZERO,
            iter: InputIter::empty(),
            consumed: 0,
            current_total_duration,
            set_current_total_duration,
            started,
            paused,
            running,
            status_update,
        }
    }

    pub fn next(&mut self) -> Option<Duration> {
        log::debug!("getting next");
        let next = self.iter.next();
        log::debug!("next = {next:?}");
        if let Some(next) = next {
            let next_duration = interpret_single(&next)
                .expect("iter should have validated duration inputs already");
            self.restart_current(next_duration);
            self.consumed += 1;
            self.current_total_duration.get_untracked()
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<Arc<str>> {
        self.iter.peek()
    }

    /// Resets the timer to its initial state, but keeping the title and input.
    pub fn reset(&mut self) {
        log::debug!("resetting timer");
        batch(|| {
            self.start_time.set(None);
            self.last_pause_time.set(None);
            (self.set_current_total_duration)(None);
        });
        self.acc_paused_duration = Duration::ZERO;
        self.iter = InputIter::empty();
        self.consumed = 0;
    }

    pub fn restart_current(&mut self, duration: Duration) {
        batch(|| {
            // force update the running memo
            // so that the interval loop restarts. restart loop to avoid being
            // highly offset from the actual time (200ms)
            self.start_time.set(None);
            self.running.get_untracked();

            (self.set_current_total_duration)(Some(duration));
            self.start_time.set(Some(relative::now()));
            self.last_pause_time.set(None);
            self.acc_paused_duration = Duration::ZERO;
        })
    }

    /// The input should be passed in by setting the `input` signal.
    pub fn start(&mut self) -> interpreter::Result<()> {
        log::debug!("starting timer with input {}", self.input.get_untracked());
        batch(|| {
            self.reset();
            // force update the running memo
            // so that the interval loop restarts. restart loop to avoid being
            // highly offset from the actual time (200ms)
            self.running.get_untracked();
            let iter = self.input.with_untracked(|input| interpret_multi(input))?;

            self.iter = iter;
            self.start_time.set(Some(relative::now()));
            self.next();
            Ok(())
        })
    }

    pub fn pause(&mut self) {
        log::debug!("paused timer");
        if self.paused.get_untracked() {
            log::debug!("timer already paused");
            return;
        }
        self.last_pause_time.set(Some(relative::now()));
    }

    pub fn resume(&mut self) {
        log::debug!("resuming timer");
        if !self.paused.get_untracked() {
            log::debug!("timer already running");
            return;
        }

        self.acc_paused_duration += relative::now()
            - self
                .last_pause_time
                .get_untracked()
                .expect("timer was paused");
        self.last_pause_time.set(None);
    }

    pub fn add_duration(&mut self, duration: Duration) {
        log::debug!("adding duration {duration} to timer");
        if !self.started.get_untracked() {
            log::warn!("timer has not started, not changing duration");
            return;
        }

        if duration.is_negative() {
            if self.finished() {
                log::warn!("timer is already finished, doing nothing");
            }
            // this binding should always work as timer is not finished and has started
            else if let Some(time_remaining) = self.time_remaining()
                && time_remaining <= -duration
            {
                // subtract will make the timer finish: saturate at 0
                let new_duration = self
                    .current_total_duration
                    .get_untracked()
                    .expect("timer should have started")
                    - time_remaining;

                log::debug!(
                    "saturating total duration to finish now: subtracted to {new_duration}"
                );

                // force timer to start overtime countdown
                self.resume();
                (self.set_current_total_duration)(Some(new_duration));
            } else {
                log::trace!("subtracting duration");
                // nothing special
                self.set_current_total_duration.update(|total| {
                    total.as_mut().map(|total| *total += duration);
                });
            }
        } else {
            if self.finished() {
                log::debug!("restarting current timer to {duration}");
                self.restart_current(duration);
            } else {
                log::trace!("adding duration");
                // nothing special
                self.set_current_total_duration.update(|total| {
                    total.as_mut().map(|total| *total += duration);
                });
            }
        }
    }

    // getters

    /// Gets the time elapsed for the current timer at the current time.
    ///
    /// This value is not reactive. Use this value whenever deriving signals,
    /// as it performs the least calculations.
    pub fn time_elapsed(&self) -> Duration {
        let Some(start_time) = self.start_time.get_untracked() else {
            log::info!("timer has not started: returning 0 duration");
            return Duration::ZERO;
        };

        let end_time = self
            .last_pause_time
            .get_untracked()
            .unwrap_or_else(relative::now);
        (end_time - start_time) - self.acc_paused_duration
    }

    /// The time remaining for the current timer at the current time.
    ///
    /// This value is not reactive.
    pub fn time_remaining(&self) -> Option<Duration> {
        self.current_total_duration
            .get_untracked()
            .map(|d| d - self.time_elapsed())
    }

    pub fn finished(&self) -> bool {
        self.time_remaining().is_some_and(|t| !t.is_positive())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}
