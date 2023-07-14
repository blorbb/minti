use leptos::*;
use std::time::Duration as StdDuration;
use time::Duration;

use crate::{
    components::{
        DurationDisplay, FullscreenButton, GrowingInput, Icon, ProgressBar, RelativeTime,
    },
    utils::{
        commands, parse, reactive,
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

    // update the time remaining when the timer is running
    reactive::repeat_while(
        cx,
        timer.running,
        move || timer.update_time_remaining(),
        StdDuration::from_millis(200),
    );

    // update the end time when the timer is paused (started and not running)
    reactive::repeat_while(
        cx,
        timer.paused,
        move || timer.update_end_time(),
        StdDuration::SECOND,
    );
    // also need to update when the timer resets,
    // so that the end time component is removed
    create_effect(cx, move |_| {
        timer.started.track();
        timer.update_end_time();
    });

    // request for user attention when the timer finishes
    create_effect(cx, move |_| {
        // also check that it is close to finish so that already expired timers
        // retrieved from localstorage don't alert
        if (timer.finished)() && timer.get_time_remaining().unwrap().abs() < Duration::SECOND {
            spawn_local(commands::alert_window());
        };
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

    let element = create_node_ref::<html::Div>(cx);

    view! { cx,
        <div
            class="com-timer"
            data-started=reactive::as_attr(timer.started)
            data-paused=reactive::as_attr(timer.paused)
            data-running=reactive::as_attr(timer.running)
            data-finished=reactive::as_attr(timer.finished)
            ref=element
        >
            <ProgressBar timer=timer/>
            <div class="timer-face">
                // stuff above the input with extra info
                <div class="heading">
                    <span class="title">
                        <GrowingInput
                            placeholder="Enter a title"
                            on_input=move |ev| timer.set_title(event_target_value(&ev))
                            initial=timer.title.get_untracked()
                        />
                    </span>

                    <Show when=move || error_message().is_some() fallback=|_| ()>
                        " | "
                        <span class="error">{error_message}</span>
                    </Show>

                    <Show when=move || end_time().is_some() fallback=|_| ()>
                        " | "
                        <span class="end">
                            <Icon icon="ph:timer-bold"/>
                            " "
                            <RelativeTime time=end_time/>
                        </span>
                    </Show>
                </div>

                // main timer display, showing either the countdown
                // or the input to enter a time
                <div class="duration">
                    <Show
                        when=timer.started
                        fallback=move |cx| {
                            view! { cx,
                                <input
                                    type="text"
                                    // set to old value when reset timer
                                    prop:value=timer.input
                                    on:input=move |ev| timer.set_input(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" {
                                            set_timer_duration();
                                        }
                                    }
                                />
                            }
                        }
                    >
                        <DurationDisplay duration=Signal::derive(
                            cx,
                            move || time_remaining().unwrap_or_default(),
                        )/>
                    </Show>
                </div>

                <div class="controls">
                    <Show
                        when=timer.started
                        fallback=move |cx| {
                            view! { cx,
                                <button class="primary" on:click=move |_| set_timer_duration()>
                                    <Icon icon="ph:play-fill"/>
                                </button>
                            }
                        }
                    >
                        // if finished, show add duration button
                        // otherwise, show add+subtract duration and pause button
                        <Show
                            when=timer.finished
                            fallback=move |cx| {
                                view! { cx,
                                    // add duration
                                    <button
                                        class="light"
                                        on:click=move |_| timer.add_duration(Duration::minutes(1))
                                    >
                                        "+ 1m"
                                    </button>
                                    <button
                                        class="light"
                                        on:click=move |_| {
                                            timer.add_duration(Duration::minutes(-1))
                                        }
                                    >
                                        "- 1m"
                                    </button>

                                    // switch between resume and pause button
                                    <Show
                                        when=timer.paused
                                        fallback=move |cx| {
                                            view! { cx,
                                                <button class="primary" on:click=move |_| timer.pause()>
                                                    <Icon icon="ph:pause-bold"/>
                                                </button>
                                            }
                                        }
                                    >
                                        <button class="primary" on:click=move |_| timer.resume()>
                                            <Icon icon="ph:play-bold"/>
                                        </button>
                                    </Show>
                                }
                            }
                        >
                            <button
                                class="primary"
                                on:click=move |_| timer.add_duration(Duration::minutes(1))
                            >
                                "+ 1m"
                            </button>
                        </Show>

                        // always show reset button
                        <button class="primary" on:click=move |_| timer.reset()>
                            <Icon icon="ph:clock-counter-clockwise-bold"/>
                        </button>

                    </Show>
                </div>

                <button class="delete" on:click=move |_| remove_self(cx, &timer)>
                    <Icon icon="ph:x-bold"/>
                </button>
                <FullscreenButton target=element/>
            </div>
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
