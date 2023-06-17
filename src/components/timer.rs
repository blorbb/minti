use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Local};
use leptos::*;

use crate::{
    components::{DurationDisplay, GrowingInput},
    utils::{parse, timer::Timer},
};

#[component]
pub fn TimerDisplay(cx: Scope, timer: Timer) -> impl IntoView {
    let time_remaining = timer.time_remaining;
    let update_time_remaining = move || timer.update_time_remaining();
    create_effect(cx, move |_| log!("updated! {:?}", time_remaining()));

    set_interval(update_time_remaining, Duration::from_millis(200));

    let (error_message, set_error_message) = create_signal(cx, None::<String>);
    let (end_time, set_end_time) = create_signal(cx, None::<DateTime<Local>>);

    let set_timer_duration = move |input: String| {
        let res = parse::parse_input(&input);

        match res {
            Ok(duration) => {
                timer.reset_with_duration(duration);
                timer.start();
                set_error_message(None);

                let now = Local::now();
                let time_to_end = ChronoDuration::from_std(duration).unwrap();
                set_end_time(Some(now + time_to_end));

                log!("inside update {:?}", (timer.duration).get_untracked());
            }
            Err(e) => {
                set_error_message(Some(e.to_string()));
            }
        }
    };


    view! { cx,
        <div class="com-timer">
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
                    " | " <span class="end">{move || format!("{:?}", end_time())}</span>
                </Show>
            </div>
            <div class="duration">
                <Show
                    when=move || {
                        (timer.started)()
                    }
                    fallback=move |cx| view! { cx,
                        <input
                            type="text"
                            on:keydown=move |ev| {
                                // log!("key {}", ev.key());
                                if ev.key() == "Enter" {
                                    let value = event_target_value(&ev);
                                    set_timer_duration(value);
                                };
                            }
                        />
                    }
                >
                    <DurationDisplay duration={time_remaining} />
                </Show>
            </div>
        </div>
    }
}
