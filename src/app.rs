use leptos::*;

use crate::{
    pages::HomePage,
    utils::timer::{serialize, TimerList},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let timers = create_rw_signal(cx, TimerList::new(cx));

    // store the timers into local storage when any of their statuses change
    create_effect(cx, move |_| {
        timers().as_vec().iter().for_each(|timer| {
            timer.timer.state_change.track();
            timer.timer.input.track();
        });

        // don't set storage if the timer list is from the signal creation above.
        if !timers().is_initial() {
            let _ = store_timers(timers());
        }
    });

    provide_context(cx, timers);

    let main_ref = create_node_ref::<html::Main>(cx);
    main_ref.on_load(cx, move |_| {
        let Some(t) = retrieve_timers(cx) else { return };
        timers.set(t);
    });

    view! { cx,
        <main ref=main_ref>
            <HomePage />
            <button on:click=move |_| timers.update(TimerList::push_new) >
                "New timer"
            </button>
            <button on:click=move |_| timers.update(TimerList::clear) >
                "Remove all"
            </button>
            <button on:click=move |_| {store_timers(timers.get_untracked());}>
                "Store timers"
            </button>
        </main>
    }
}

// TODO handle errors properly
fn store_timers(timers: TimerList) -> Option<()> {
    let local_storage = window().local_storage().ok()??;
    let timers_string = serialize::stringify_timers(timers);
    local_storage.set_item("timers", &timers_string).ok()?;
    Some(())
}

fn retrieve_timers(cx: Scope) -> Option<TimerList> {
    let local_storage = window().local_storage().ok()??;
    let timers_string = local_storage.get_item("timers").ok()??;
    serialize::parse_timer_json(cx, &timers_string)
}
