use leptos::*;

use crate::{components::timer::TimerDisplay, utils::timer::TimerList};

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let timers = expect_context::<RwSignal<TimerList>>(cx);

    view! { cx,
        <div class="page-home">
            <For
                each=timers
                key=|timer| timer.id
                view=move |cx, timer| view! { cx,
                    <TimerDisplay timer={(timer.timer)()} />
                }
            />
        </div>
    }
}
