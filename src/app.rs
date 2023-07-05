use leptos::*;

use crate::{
    pages::HomePage,
    utils::timer::{serialize, TimerList},
};

/// Main application component that manages global state.
///
/// - Provides a context `RwSignal<TimerList>` to all descendants.
/// - Updates localstorage whenever a timer changes.
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let timers = create_rw_signal(cx, TimerList::new(cx));
    provide_context(cx, timers);

    // TODO state changes are not being tracked
    // store the timers into local storage when any of their statuses change
    create_effect(cx, move |_| {
        timers().as_vec().iter().for_each(|timer| {
            timer.state_change.track();
            timer.input.track();
        });

        // don't set storage if the timer list is from the signal creation above.
        if !timers().is_initial() {
            let _ = store_timers(timers());
        }
    });

    // load timers from localstorage
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
/// Stores a `TimerList` into localstorage.
///
/// The timers will be stored using the key "timers".
///
/// Returns `None` if localstorage cannot be accessed or it failed to set the item.
fn store_timers(timers: TimerList) -> Option<()> {
    let local_storage = window().local_storage().ok()??;
    let timers_string = serialize::stringify_timers(timers);
    local_storage.set_item("timers", &timers_string).ok()?;
    Some(())
}

/// Retrieves timers from localstorage and sets them in the correct state.
///
/// The timers are expected to be in the key "timers".
///
/// Returns `None` if localstorage cannot be accessed or the item cannot be parsed.
fn retrieve_timers(cx: Scope) -> Option<TimerList> {
    let local_storage = window().local_storage().ok()??;
    let timers_string = local_storage.get_item("timers").ok()??;
    serialize::parse_timer_json(cx, &timers_string)
}
