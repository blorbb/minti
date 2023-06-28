use leptos::*;

use crate::{components::TimerDisplay, utils::timer::{TimerList, Timer}};

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let timers = expect_context::<RwSignal<TimerList>>(cx);

    view! { cx,
        <div class="page-home">
            <For
                each=timers
                key=Timer::id
                view=move |cx, timer| view! { cx,
                    <TimerDisplay timer=timer />
                }
            />
        </div>
    }
}
