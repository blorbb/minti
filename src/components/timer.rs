use leptos::*;
use std::time::Duration as StdDuration;
use time::Duration;
use wasm_bindgen::JsValue;

use crate::{
    components::{
        DurationDisplay, FullscreenButton, GrowingInput, Icon, ProgressBar, RelativeTime,
    },
    utils::{commands, contexts::TimerList, parse, reactive, timer::Timer},
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
        if (timer.finished)()
            && timer
                .get_time_remaining()
                .expect("timer is finished => should have started")
                .abs()
                < Duration::SECOND
        {
            spawn_local(commands::alert_window());
        };
    });

    let set_timer_duration = move || {
        let res = parse::parse_input(&timer.input.get_untracked());

        match res {
            Ok(duration) => {
                timer.restart(duration);
                set_error_message(None);
            }
            Err(e) => {
                set_error_message(Some(e.to_string()));
            }
        }
    };

    let component = create_node_ref::<html::Div>(cx);
    let duration_display = create_node_ref::<html::Div>(cx);

    let update_timer_duration =
        move |duration: Duration| update_and_bump(duration, duration_display, &timer);

    // sub-components //

    // switch between resume and pause button
    let pause_button = move || {
        if (timer.paused)() {
            view! { cx,
                <button class="primary mix-btn-scale-green" on:click=move |_| timer.resume()>
                    <Icon icon="ph:play-bold"/>
                </button>
            }
        } else {
            view! { cx,
                <button class="primary mix-btn-scale-green" on:click=move |_| timer.pause()>
                    <Icon icon="ph:pause-bold"/>
                </button>
            }
        }
    };

    let controls_start = move || {
        view! { cx,
            <button class="primary mix-btn-scale-green" on:click=move |_| set_timer_duration()>
                <Icon icon="ph:play-fill"/>
            </button>
        }
    };

    let controls_running = move || {
        view! { cx,
            // add duration
            <button
                class="light mix-btn-transp-neutral"
                on:click=move |_| update_timer_duration(Duration::MINUTE)
            >
                "+ 1m"
            </button>
            <button
                class="light mix-btn-transp-neutral"
                on:click=move |_| update_timer_duration(-Duration::MINUTE)
            >
                "- 1m"
            </button>

            {pause_button}

            <button class="primary mix-btn-scale-green" on:click=move |_| timer.reset()>
                <Icon icon="ph:clock-counter-clockwise-bold"/>
            </button>
        }
    };

    let controls_finished = move || {
        view! { cx,
            <button
                class="primary mix-btn-scale-green"
                on:click=move |_| timer.add_duration(Duration::minutes(1))
            >
                "+ 1m"
            </button>

            <button class="primary mix-btn-scale-green" on:click=move |_| timer.reset()>
                <Icon icon="ph:clock-counter-clockwise-bold"/>
            </button>
        }
    };

    // using <Show /> causes components to re-render for some reason
    // using `if` is fine as `started` and `finished` are memos anyways.
    let controls = move || {
        if !(timer.started)() {
            controls_start().into_view(cx)
        } else if !(timer.finished)() {
            controls_running().into_view(cx)
        } else {
            controls_finished().into_view(cx)
        }
    };

    view! { cx,
        <div
            class="com-timer"
            data-started=reactive::as_attr(timer.started)
            data-paused=reactive::as_attr(timer.paused)
            data-running=reactive::as_attr(timer.running)
            data-finished=reactive::as_attr(timer.finished)
            ref=component
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
                <div class="duration" ref=duration_display>
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
                    {controls}
                </div>

                <button class="delete mix-btn-transp-red" on:click=move |_| remove_self(cx, &timer)>
                    <Icon icon="ph:x-bold"/>
                </button>
                <FullscreenButton class="mix-btn-transp-neutral" target=component/>
            </div>
        </div>
    }
}

fn remove_self(cx: Scope, timer: &Timer) {
    let timers = expect_context::<TimerList>(cx);
    timers.remove_id(timer.id());
    cx.dispose();
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
