use std::time::Duration;

use leptos::{leptos_dom::helpers::IntervalHandle, *};

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

    // update the time remaining when the timer is running
    repeat_while(
        cx,
        timer.running,
        move || {
            log!("updating time rem");
            timer.update_time_remaining();
        },
        Duration::from_millis(200),
    );

    // update the end time when the timer is paused (started and not running)
    repeat_while(
        cx,
        timer.paused,
        move || {
            log!("updating end");
            timer.update_end_time();
        },
        Duration::SECOND,
    );
    // also need to update when the timer resets,
    // so that the end time component is removed
    create_effect(cx, move |_| {
        if !(timer.started)() {
            timer.update_end_time();
        };
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

/// Runs a callback and repeats it while `when` is true.
fn repeat_while(
    cx: Scope,
    when: impl Fn() -> bool + 'static,
    callback: impl Fn() + Clone + 'static,
    duration: Duration,
) {
    // needs double Option as the outer one is None on first run,
    // but needs to be None if when() is false.
    #[expect(clippy::option_option, reason = "required")]
    create_effect(cx, move |prev_handle: Option<Option<IntervalHandle>>| {
        // cancel the previous handle if it exists
        if let Some(prev_handle) = prev_handle.flatten() {
            prev_handle.clear();
        };

        if when() {
            callback();
            Some(
                set_interval_with_handle(callback.clone(), duration)
                    .expect("Could not create interval"),
            )
        } else {
            None
        }
        // return handle so that next call can access it
    });
}
