use leptos::*;
use web_sys::Element;

use crate::{
    components::Icon,
    pages::HomePage,
    utils::timer::{serialize, TimerList},
};

#[derive(Debug, Clone)]
pub struct FullscreenElement(pub Option<Element>);

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
            timer.title.track();
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

    // set fullscreen element context
    let (fullscreen_element, set_fullscreen_element) = create_signal(cx, FullscreenElement(None));
    window_event_listener(ev::fullscreenchange, move |_| {
        set_fullscreen_element(FullscreenElement(document().fullscreen_element()));
    });
    provide_context(cx, fullscreen_element);

    view! { cx,
        <div class="page">
            <main ref=main_ref>
                <HomePage />
            </main>
            <nav>
                <button class="add" on:click=move |_| timers.update(TimerList::push_new) >
                    <Icon icon="ph:plus-bold" />
                </button>
                <button class="remove" on:click=move |_| timers.update(TimerList::clear) >
                    <Icon icon="ph:trash-bold" />
                </button>
            </nav>
        </div>
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
