use leptos::*;
use leptos_mview::mview;

use time::Duration;
use wasm_bindgen::JsValue;

use crate::{
    components::{
        DurationDisplay, DurationUpdateButton, FullscreenButton, GrowingInput, Icon, ProgressBar,
        RelativeTime,
    },
    contexts::TimerList,
    parse, reactive,
    timer::Timer,
};

/// Provides controls and display for a [`Timer`].
#[component]
pub fn TimerDisplay(timer: Timer) -> impl IntoView {
    let (error_message, set_error_message) = create_signal(None::<String>);

    let set_timer_duration = move || match parse::parse_input(&timer.input().get_untracked()) {
        Ok(duration) => {
            timer.restart(duration);
            set_error_message(None);
        }
        Err(e) => {
            set_error_message(Some(e.to_string()));
        }
    };

    let component = create_node_ref::<html::Div>();
    let duration_display = create_node_ref::<html::Div>();

    let update_timer_duration =
        move |duration: Duration| update_and_bump(duration, duration_display, &timer);

    // sub-components //

    // switch between resume and pause button
    let pause_button = move || {
        if (timer.paused())() {
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
            button.primary.mix-btn-scale-green on:click={move |_| set_timer_duration()} {
                Icon icon="ph:play-fill";
            }
        }
    };

    let controls_running = move || {
        mview! {
            DurationUpdateButton
                button_class="light mix-btn-transp-neutral"
                on_click={update_timer_duration}
                add;
            DurationUpdateButton
                button_class="light mix-btn-transp-neutral"
                on_click={update_timer_duration}
                add=false;

            {pause_button}

            button.primary.mix-btn-scale-green on:click={move |_| timer.reset()} {
                Icon icon="ph:clock-counter-clockwise-bold";
            }
        }
    };

    let controls_finished = move || {
        mview! {
            DurationUpdateButton
                button_class="primary mix-btn-scale-green"
                on_click={update_timer_duration}
                add;

            button.primary.mix-btn-scale-green on:click={move |_| timer.reset()} {
                Icon icon="ph:clock-counter-clockwise-bold";
            }
        }
    };

    // using <Show /> causes components to re-render for some reason
    // using `if` is fine as `started` and `finished` are memos anyways.
    let controls = move || {
        if !(timer.started())() {
            controls_start().into_view()
        } else if !(timer.finished())() {
            controls_running().into_view()
        } else {
            controls_finished().into_view()
        }
    };

    mview! {
        div.com-timer
            data-started={reactive::as_attr(timer.started())}
            data-paused={reactive::as_attr(timer.paused())}
            data-running={reactive::as_attr(timer.running())}
            data-finished={reactive::as_attr(timer.finished())}
            ref={component}
        {
            ProgressBar {timer};
            div.timer-face {
                // stuff above the input with extra info
                div.heading {
                    span.title {
                        GrowingInput
                            placeholder="Enter a title"
                            on_input={move |ev| timer.set_title(event_target_value(&ev))}
                            initial={timer.title().get_untracked()};
                    }

                    Show when=[error_message().is_some()] {
                        " | "
                        span.error { {error_message} }
                    }

                    Show when=[(timer.end_time())().is_some()] {
                        " | "
                        span.end {
                            Icon icon="ph:timer-bold";
                            " "
                            RelativeTime time={timer.end_time()};
                        }
                    }
                }
                // main timer display, showing either the countdown
                // or the input to enter a time
                div.duration ref={duration_display} {
                    [if (timer.started())() {
                        mview! {
                            DurationDisplay duration={
                                Signal::derive(move || (timer.time_remaining())().unwrap_or_default())
                            };
                        }.into_view()
                    } else {
                        mview! {
                            input
                                type="text"
                                // set old value when reset timer
                                prop:value={timer.input()}
                                on:input={move |ev| timer.set_input(event_target_value(&ev))}
                                on:keydown={move |ev| {
                                    if ev.key() == "Enter" {
                                        set_timer_duration();
                                    }
                                }};
                        }.into_view()
                    }]
                }

                div.controls { {controls} }

                button.delete.mix-btn-transp-red on:click={move |_| remove_self(&timer)} {
                    Icon icon="ph:x-bold";
                }
                FullscreenButton class="mix-btn-transp-neutral" target={component};
            }
        }
    }
}

fn remove_self(timer: &Timer) {
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
fn update_and_bump(duration: Duration, element: NodeRef<html::Div>, timer: &Timer) {
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
