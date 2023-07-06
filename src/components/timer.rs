use leptos::*;
use std::time::Duration as StdDuration;

use crate::{
    components::{DurationDisplay, GrowingInput, Icon, RelativeTime},
    utils::{
        parse, reactive,
        timer::{Timer, TimerList},
    },
};

/// Provides controls and display for a [`Timer`].
#[expect(clippy::too_many_lines, reason = "idk how make smaller")]
#[expect(clippy::large_types_passed_by_value, reason = "can't be reference")]
#[component]
pub fn TimerDisplay(cx: Scope, timer: Timer) -> impl IntoView {
    let time_remaining = timer.time_remaining;
    let end_time = timer.end_time;
    let (error_message, set_error_message) = create_signal(cx, None::<String>);

    // TODO!! not sure if timer is being disposed of properly. maybe it should be a rwsignal?
    on_cleanup(cx, move || timer.reset());

    // update the time remaining when the timer is running
    reactive::repeat_while(
        cx,
        timer.running,
        move || {
            log!("updating time rem");
            timer.update_time_remaining();
        },
        StdDuration::from_millis(200),
    );

    // update the end time when the timer is paused (started and not running)
    reactive::repeat_while(
        cx,
        timer.paused,
        move || {
            log!("updating end");
            timer.update_end_time();
        },
        StdDuration::SECOND,
    );
    // also need to update when the timer resets,
    // so that the end time component is removed
    create_effect(cx, move |_| {
        timer.started.track();
        timer.update_end_time();
    });

    let set_timer_duration = move || {
        let res = parse::parse_input(&timer.input.get_untracked());

        match res {
            Ok(duration) => cx.batch(|| {
                timer.reset();
                timer.start(duration);
                set_error_message(None);
            }),
            Err(e) => {
                set_error_message(Some(e.to_string()));
            }
        }
    };

    view! { cx,
        <div class="com-timer"
            data-started=reactive::as_attr(timer.started)
            data-paused=reactive::as_attr(timer.paused)
            data-running=reactive::as_attr(timer.running)
            data-finished=reactive::as_attr(timer.finished)
        >
            // stuff above the input with extra info
            <div class="heading">
                <span class="title">
                    <GrowingInput placeholder="Enter a title"/>
                </span>

                <Show
                    when=move || error_message().is_some()
                    fallback=|_| ()
                >
                    " | " <span class="error">{error_message}</span>
                </Show>

                <Show
                    when=move || end_time().is_some()
                    fallback=|_| ()
                >
                    " | " <span class="end">
                        <Icon icon="ph:timer-bold" />" "
                        <RelativeTime time=end_time />
                    </span>
                </Show>
            </div>

            // main timer display, showing either the countdown
            // or the input to enter a time
            <div class="duration">
                <Show
                    when=timer.started
                    fallback=move |cx| view! { cx,
                        <input
                            type="text"
                            prop:value=timer.input // set to old value when reset timer
                            on:input=move |ev| (timer.set_input)(event_target_value(&ev))
                            on:keydown=move |ev| {
                                // log!("key {}", ev.key());
                                if ev.key() == "Enter" {
                                    set_timer_duration();
                                };
                            }
                        />
                    }
                >
                    <DurationDisplay duration={Signal::derive(cx, move || time_remaining().unwrap_or_default())} />
                </Show>
            </div>

            <div class="controls">
                <Show
                    when=timer.started
                    fallback=move |cx| view! { cx,
                        <button on:click=move |_| set_timer_duration()>
                            // TODO: this creates a warning that a signal is updated
                            // after being disposed.
                            <Icon icon="ph:play-fill"/>
                        </button>
                    }
                >
                    // switch between resume and pause button
                    <button
                        on:click=move |_| if (timer.paused)() {
                            timer.resume();
                        } else {
                            timer.pause();
                        }
                    >
                        <Icon
                            icon=Signal::derive(
                                cx,
                                move || if (timer.paused)() {
                                    "ph:play-bold"
                                } else {
                                    "ph:pause-bold"
                                }
                            )
                        />
                    </button>
                    <button on:click=move |_| timer.reset()>
                        <Icon icon="ph:clock-counter-clockwise-bold"/>
                    </button>
                </Show>
            </div>

            <button
                class="delete"
                on:click=move |_| remove_self(cx, &timer)
            >
                <Icon icon="ph:x-bold"/>
            </button>
        </div>
    }
}

fn remove_self(cx: Scope, timer: &Timer) {
    let timers = expect_context::<RwSignal<TimerList>>(cx);
    timers.update(|t| {
        t.remove_id(timer.id());
    });
    cx.dispose();
}
