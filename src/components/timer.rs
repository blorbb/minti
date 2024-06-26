use leptos::*;
use leptos_mview::mview;

use leptos_use::{storage::use_local_storage, utils::FromToStringCodec};
use std::time::Duration as StdDuration;
use time::Duration;
use wasm_bindgen::JsValue;

use crate::{
    commands,
    components::{
        DurationDisplay, DurationUpdateButton, FullscreenButton, GrowingInput, Icon, ProgressBar,
        RelativeTime,
    },
    contexts::TimerList,
    reactive,
    time::relative,
    timer::MultiTimer,
};

/// Provides controls and display for a [`Timer`].
#[component]
pub fn TimerDisplay(timer: MultiTimer) -> impl IntoView {
    let (error_message, set_error_message) = create_signal(None::<String>);

    let update_time_elapsed = Trigger::new();
    let time_elapsed = Memo::new(move |_| {
        update_time_elapsed.track();
        timer.time_elapsed()
    });
    let time_remaining =
        Memo::new(move |_| timer.current_total_duration()().map(|t| t - time_elapsed()));
    let finished = Memo::new(move |_| time_remaining().is_some_and(|t| !t.is_positive()));

    // another signal instead of a memo to avoid highly frequent updates
    let update_end_time = Trigger::new();
    let end_time = Memo::new(move |_| time_remaining().map(|t| relative::now() + t));

    reactive::repeat_while(
        timer.running(),
        move || update_time_elapsed.notify(),
        StdDuration::from_millis(200),
    );
    reactive::repeat_while(
        timer.paused(),
        move || update_end_time.notify(),
        StdDuration::SECOND,
    );
    Effect::new(move |_| {
        timer.status_update().track();
        update_time_elapsed.notify();
        update_end_time.notify();
    });

    Effect::new(move |_| {
        // also check that it is close to finish so that already expired timers
        // retrieved from localstorage don't alert
        if finished()
            && time_remaining
                .get_untracked()
                .expect("timer is finished => should have started")
                .abs()
                < Duration::SECOND
        {
            spawn_local(commands::alert_window());
        };
    });

    let peek = RwSignal::new(timer.peek());

    let component = NodeRef::<html::Div>::new();
    let duration_display = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        if finished() {
            timer.next();
            peek.set(timer.peek());
            flash(duration_display)
        }
    });

    let start = move || match timer.start() {
        Ok(_) => {
            set_error_message(None);
            peek.set(timer.peek());
        }
        Err(e) => {
            set_error_message(Some(e.to_string()));
        }
    };

    let update_timer_duration =
        move |duration: Duration| update_and_bump(duration, duration_display, timer);

    // sub-components //

    let next_time = move || {
        mview! {
            Show when=[peek().is_some() && timer.started()()] {
                div.next-timer {
                   span("next:")
                   span({peek().map(|s| s.to_string())})
                }
            }
        }
    };

    // switch between resume and pause button
    let pause_button = move || {
        if timer.paused()() {
            mview! {
                button.primary.mix-btn-scale-green on:click={move |_| timer.resume()} {
                    Icon icon="ph:play-bold";
                }
            }
        } else {
            mview! {
                button.primary.mix-btn-scale-green on:click={move |_| timer.pause()} {
                    Icon icon="ph:pause-bold";
                }
            }
        }
    };

    let controls_start = move || {
        mview! {
            button.primary.mix-btn-scale-green on:click={move |_| start()} {
                Icon icon="ph:play-fill";
            }
        }
    };

    let controls_running = move || {
        mview! {
            DurationUpdateButton
                button_class="mix-btn-transp-neutral"
                on_click={update_timer_duration};

            {pause_button}

            button.primary.mix-btn-scale-green on:click={move |_| timer.reset()} {
                Icon icon="ph:clock-counter-clockwise-bold";
            }
        }
    };

    let controls_finished = move || {
        mview! {
            DurationUpdateButton
                button_class="mix-btn-transp-neutral"
                on_click={update_timer_duration}
                add-only;

            button.primary.mix-btn-scale-green on:click={move |_| timer.reset()} {
                Icon icon="ph:clock-counter-clockwise-bold";
            }
        }
    };

    // using <Show /> causes components to re-render for some reason
    // using `if` is fine as `started` and `finished` are memos anyways.
    let controls = move || {
        if !timer.started()() {
            controls_start().into_view()
        } else if !finished() {
            controls_running().into_view()
        } else {
            controls_finished().into_view()
        }
    };

    let time_elapsed = move || {
        timer.current_total_duration()().unwrap_or_default()
        - time_remaining().unwrap_or_default()
        // make the digit round down, but +1ms to avoid showing -1s at the start
        - Duration::SECOND
            + Duration::MILLISECOND
    };

    let (show_heading_title, _, _) =
        use_local_storage::<bool, FromToStringCodec>("heading-show::title");
    let (show_heading_end_time, _, _) =
        use_local_storage::<bool, FromToStringCodec>("heading-show::end-time");
    let (show_heading_elapsed, _, _) =
        use_local_storage::<bool, FromToStringCodec>("heading-show::elapsed");
    let show_heading_end_time = Memo::new(move |_| show_heading_end_time() && end_time().is_some());
    let show_heading_elapsed =
        Memo::new(move |_| show_heading_elapsed() && timer.started()() && !finished());

    let heading_views = [
        mview! {
            span.title {
                GrowingInput
                    placeholder="Enter a title"
                    on_input={move |ev| timer.title().set(event_target_value(&ev))}
                    initial={timer.title().get_untracked()};
            }
        }
        .into_view(),
        mview! {
            span.end {
                Icon icon="ph:timer-bold";
                " "
                RelativeTime time={end_time};
            }
        }
        .into_view(),
        mview! {
            span.elapsed {
                DurationDisplay
                    duration={time_elapsed};
            }
        }
        .into_view(),
        // this will only be shown with the title, if it exists
        mview! {
            span.error { {error_message} }
        }
        .into_view(),
    ];

    // filter to only show the views that have been enabled,
    // with a " | " separator between each one
    let heading = Memo::new(move |_| {
        heading_views
            .iter()
            .enumerate()
            .filter(move |(i, _)| {
                [
                    show_heading_title(),
                    show_heading_end_time(),
                    show_heading_elapsed(),
                    error_message().is_some(),
                ][*i]
            })
            .map(|view| view.1)
            .cloned()
            .intersperse_with(|| " | ".into_view())
            .collect_view()
    });

    mview! {
        div.com-timer
            data-started={reactive::as_attr(timer.started())}
            data-paused={reactive::as_attr(timer.paused())}
            data-running={reactive::as_attr(timer.running())}
            data-finished={reactive::as_attr(finished)}
            ref={component}
        {
            ProgressBar {timer} {finished};
            div.timer-face {
                // stuff above the input with extra info
                div.heading {
                    {heading}
                }

                // main timer display, showing either the countdown
                // or the input to enter a time
                div.middle {
                    div.duration ref={duration_display} {
                        [if timer.started()() {
                            mview! {
                                DurationDisplay duration=[
                                    time_remaining().unwrap_or_default()
                                ];
                            }.into_view()
                        } else {
                            mview! {
                                input
                                    type="text"
                                    // set old value when reset timer
                                    prop:value={timer.input()}
                                    on:input={move |ev| timer.input().set(event_target_value(&ev))}
                                    on:keydown={move |ev| {
                                        if ev.key() == "Enter" {
                                            start();
                                        }
                                    }};
                            }.into_view()
                        }]
                    }

                    {next_time}
                }

                div.controls { {controls} }


                button.delete.mix-btn-transp-red on:click={move |_| remove_self(timer)} {
                    Icon icon="ph:x-bold";
                }
                FullscreenButton class="mix-btn-transp-neutral" target={component};
            }
        }
    }
}

fn remove_self(timer: MultiTimer) {
    let timers = expect_context::<TimerList>();
    timers.remove_id(timer.id());
}

/// Creates a JS object with one key-value pair.
fn js_obj_1(key: &str, value: &str) -> js_sys::Object {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &key.into(), &value.into()).unwrap();
    obj
}

/// Updates the timer's duration and bumps the element up/down
fn update_and_bump(duration: Duration, element: NodeRef<html::Div>, timer: MultiTimer) {
    let mut anim_options = web_sys::KeyframeAnimationOptions::new();
    anim_options.duration(&JsValue::from_f64(100.0));
    anim_options.easing("ease-out");

    timer.add_duration(duration);

    if let Some(display) = element.get_untracked() {
        if duration.is_positive() {
            let anim_up_keyframes: js_sys::Array = [
                js_obj_1("transform", "translateY(0)"),
                js_obj_1("transform", "translateY(-0.2em)"),
                js_obj_1("transform", "translateY(0)"),
            ]
            .into_iter()
            .collect();
            display
                .animate_with_keyframe_animation_options(Some(&anim_up_keyframes), &anim_options);
        } else {
            let anim_down_keyframes: js_sys::Array = [
                js_obj_1("transform", "translateY(0)"),
                js_obj_1("transform", "translateY(0.2em)"),
                js_obj_1("transform", "translateY(0)"),
            ]
            .into_iter()
            .collect();
            display
                .animate_with_keyframe_animation_options(Some(&anim_down_keyframes), &anim_options);
        };
    };
}

fn flash(element: NodeRef<html::Div>) {
    let Some(display) = element.get_untracked() else {
        return;
    };

    let mut anim_options = web_sys::KeyframeAnimationOptions::new();
    anim_options.duration(&JsValue::from_f64(480.0));
    anim_options.easing("steps(5, end)");

    // makes 3 red flashes whether it ends white or ends red
    let anim_up_keyframes: js_sys::Array = [
        js_obj_1("color", "var(--clr-red-400)"),
        js_obj_1("color", "var(--clr-text)"),
        js_obj_1("color", "var(--clr-red-400)"),
        js_obj_1("color", "var(--clr-text)"),
        js_obj_1("color", "var(--clr-red-400)"),
        js_obj_1("color", "var(--clr-text)"),
    ]
    .into_iter()
    .collect();
    display.animate_with_keyframe_animation_options(Some(&anim_up_keyframes), &anim_options);
}
