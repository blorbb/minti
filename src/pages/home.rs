use leptos::*;

use crate::{
    components::TimerDisplay,
    utils::{contexts::TimerList, timer::Timer},
};

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let timers = expect_context::<TimerList>(cx);

    view! { cx,
        <div class="page-home">
            <For
                each=timers.vec_signal()
                key=Timer::id
                view=move |cx, timer| view! { cx, <TimerDisplay timer=timer/> }
            />
        </div>
    }
}
