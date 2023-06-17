use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Local};
use leptos::*;

use crate::{
    components::{DurationDisplay, GrowingInput, RelativeTime},
    utils::{parse, timer::Timer},
};

#[component]
pub fn TimerDisplay(cx: Scope, timer: Timer) -> impl IntoView {
    let time_remaining = timer.time_remaining;
    let update_time_remaining = move || timer.update_time_remaining();

    let (error_message, set_error_message) = create_signal(cx, None::<String>);
    let (end_time, set_end_time) = create_signal(cx, None::<DateTime<Local>>);

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

    create_effect(cx, move |_| {
        log!("running {}", (timer.running)());
        if (timer.running)() {
            countdown_handle.get_untracked().clear();
            end_time_handle.get_untracked().clear();
            countdown_handle.set(
                set_interval_with_handle(update_time_remaining, Duration::from_millis(200))
                    .expect("something went wrong with setting interval"),
            );
        } else {
            countdown_handle.get_untracked().clear();
            if (timer.started)() {
                end_time_handle.get_untracked().clear();
                end_time_handle.set(
                    set_interval_with_handle(
                        move || set_end_time(Some(Local::now())),
                        Duration::SECOND,
                    )
                    .expect("Something went wrong setting end time handle"),
                )
            }
        }
    });

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
                    " | " <span class="end">
                    <RelativeTime time=end_time />
                    </span>
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
