use leptos::*;
use leptos_mview::view;

use crate::{
    components::TimerDisplay,
    utils::{contexts::TimerList, timer::Timer},
};

#[component]
pub fn HomePage() -> impl IntoView {
    let timers = expect_context::<TimerList>();

    view! {
        div.page-home {
            For
                each={timers.vec_signal()}
                key={Timer::id}
                view={move |timer| view! { TimerDisplay {timer}; }};
        }
    }
}
