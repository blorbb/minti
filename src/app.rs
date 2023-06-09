use leptos::*;

use crate::{pages::home::HomePage, utils::timer::TimerList};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let timers = create_rw_signal(cx, TimerList::new(cx, 1));
    create_effect(cx, move |_| log!("{:?}", timers().0.iter().map(|t| t.timer().duration).collect::<Vec<_>>()));

    provide_context(cx, timers);

    view! { cx,
        <main>
            <HomePage />
        </main>
    }
}