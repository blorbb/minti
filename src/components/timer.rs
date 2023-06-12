use std::time::Duration;

use leptos::*;

use crate::{components::duration::DurationDisplay, utils::timer::Timer};

#[component]
pub fn TimerDisplay(cx: Scope, timer: Timer) -> impl IntoView {
    let time_remaining = timer.time_remaining;
    let update_time_remaining = move || timer.update_time_remaining();
    create_effect(cx, move |_| log!("updated! {:?}", time_remaining()));

    set_interval(update_time_remaining, Duration::from_millis(200));

    let set_timer_duration = move |secs: u64| {
        timer.reset_with_duration(Duration::from_secs(secs));
        timer.start();
        log!("inside update {:?}", (timer.duration).get_untracked());
    };

    view! { cx,
        <div class="com-timer">
            <div class="duration">
                <Show
                    when=move || {
                        (timer.started)()
                    }
                    fallback=move |cx| view! { cx,
                        <input
                            type="number"
                            on:keydown=move |ev| {
                                // log!("key {}", ev.key());
                                if ev.key() == "Enter" {
                                    let value = event_target_value(&ev).parse::<u64>().unwrap();
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
