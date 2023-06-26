use std::time::Duration;

use leptos::*;

use crate::{
    components::{DurationDisplay, GrowingInput, Icon, RelativeTime},
    utils::{parse, timer::Timer},
};

#[expect(clippy::too_many_lines, reason = "idk how make smaller")]
#[expect(clippy::large_types_passed_by_value, reason = "can't be reference")]
#[component]
pub fn TimerDisplay(cx: Scope, timer: Timer) -> impl IntoView {
    let time_remaining = timer.time_remaining;
    let end_time = timer.end_time;
    let (error_message, set_error_message) = create_signal(cx, None::<String>);

    let countdown_handle = create_rw_signal(
        cx,
        set_interval_with_handle(|| (), Duration::SECOND)
            .expect("something went wrong with setting interval"),
    );
    let end_time_handle = create_rw_signal(
        cx,
        set_interval_with_handle(|| (), Duration::SECOND)
            .expect("Something went wrong setting end time handle"),
    );

    // updates the countdown and end times when the timer's `running` status changes.
    create_effect(cx, move |_| {
        log!("running {}", (timer.running)());
        countdown_handle.get_untracked().clear();
        end_time_handle.get_untracked().clear();

        // update the countdown if the timer is running
        // update the end time otherwise
        if (timer.running)() {
            timer.update_time_remaining();
            countdown_handle.set(
                set_interval_with_handle(
                    move || timer.update_time_remaining(),
                    Duration::from_millis(200),
                )
                .expect("something went wrong with setting interval"),
            );
        } else {
            timer.update_end_time();
            end_time_handle.set(
                set_interval_with_handle(move || timer.update_end_time(), Duration::SECOND)
                    .expect("Something went wrong setting end time handle"),
            );
        }
    });

    let set_timer_duration = move || {
        let res = parse::parse_input(&timer.input.get_untracked());

        match res {
            Ok(duration) => {
                timer.reset_with_duration(duration);
                timer.start();
                set_error_message(None);
                timer.update_end_time();
            }
            Err(e) => {
                set_error_message(Some(e.to_string()));
            }
        }
    };

    view! { cx,
        <div class="com-timer"
            data-started={move || (timer.started)().to_string()}
            data-paused={move || (timer.paused)().to_string()}
            data-running={move || (timer.running)().to_string()}
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
                        <Icon inline=true icon="ph:timer-bold" />" "
                        <RelativeTime time=end_time />
                    </span>
                </Show>
            </div>

            // main timer display, showing either the countdown
            // or the input to enter a time
            <div class="duration">
                <Show
                    when=move || {
                        (timer.started)()
                    }
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
                    <DurationDisplay duration={time_remaining} />
                </Show>
            </div>

            <div class="controls">
                <Show
                    when=timer.started
                    fallback=move |cx| view! { cx,
                        <button on:click=move |_| set_timer_duration()>
                            // TODO: this creates a warning that a signal is updated
                            // after being disposed.
                            <Icon inline=true icon="ph:play-fill"/>
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
                            inline=true
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
                        <Icon inline=true icon="ph:clock-counter-clockwise-bold"/>
                    </button>
                </Show>
            </div>
        </div>
    }
}
