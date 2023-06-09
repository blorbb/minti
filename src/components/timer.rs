use std::time::Duration;

use leptos::*;
use uuid::Uuid;

use crate::{components::duration::DurationDisplay, utils::timer::TimerList};

#[component]
pub fn TimerDisplay(cx: Scope, timer_id: Uuid) -> impl IntoView {
    let timers = expect_context::<RwSignal<TimerList>>(cx);

    let timer = move || {
        let binding = timers();
        let timer = binding.timer_with_id(timer_id);
        log!("{:?}, {:?}", (timer.started)(), timer.duration);
        timers().timer_with_id(timer_id).clone()
    };

    let time_remaining = create_rw_signal(cx, timer().time_remaining());

    // set_interval(
    //     move || {
    //         log!("{:?}", timer().time_remaining());
    //         time_remaining.set(timer().time_remaining());
    //     },
    //     Duration::SECOND,
    // );

    let set_timer_duration = move |duration: u64| {
        log!("updated here! {}", duration);
        log!("1: {:?}", timers().timer_with_id(timer_id).duration);
        timers.update(|list| {
            log!("IAOHWROAWIKHR");
            list.0[0]
                .timer()
                .reset_with_duration(Duration::from_secs(10));
        });
        log!("2: {:?}", timers().timer_with_id(timer_id).duration);
    };

    view! { cx,
        <div class="com-timer">
            <div class="duration">
                <Show
                    when=move || {
                        (timer().started)()
                    }
                    fallback=move |cx| view! { cx,
                        <input
                            type="number"
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" {
                                    set_timer_duration(event_target_value(&ev).parse().unwrap());
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
