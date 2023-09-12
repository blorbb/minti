use js_sys::Function;
use leptos::*;
use leptos_mview::view;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    HtmlElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
};

use crate::{
    components::Icon,
    pages::HomePage,
    utils::{
        contexts::{FullscreenElement, Icons, TimerList},
        timer::serialize,
    },
};

/// Main application component that manages global state.
///
/// - Provides a context `RwSignal<TimerList>` to all descendants.
/// - Updates localstorage whenever a timer changes.
#[component]
pub fn App() -> impl IntoView {
    let timers = TimerList::new();
    provide_context(timers);

    // load timers from localstorage
    let main_ref = create_node_ref::<html::Main>();
    main_ref.on_load(move |_| {
        let Some(t) = retrieve_timers() else { return };
        timers.set(t.to_vec());
    });

    // store timers into localstorage before unload
    window_event_listener(ev::beforeunload, move |_| {
        if !timers.is_initial() {
            log::debug!("storing timers");
            let _ = store_timers(timers);
        }
    });

    // contexts //

    // fullscreen element context
    let (fullscreen_element, set_fullscreen_element) = create_signal(None);
    let fullscreen_element = FullscreenElement::new(fullscreen_element);

    window_event_listener(ev::fullscreenchange, move |_| {
        set_fullscreen_element(document().fullscreen_element());
    });

    provide_context(fullscreen_element);

    // icons context
    let icons = Icons::from_local_storage();
    provide_context(icons);

    // show scroll shadows //

    let intersection_root = create_node_ref::<html::Div>();
    let top_edge = create_node_ref::<html::Div>();
    let bottom_edge = create_node_ref::<html::Div>();
    let top_shadow = create_node_ref::<html::Div>();
    let bottom_shadow = create_node_ref::<html::Div>();

    bottom_shadow.on_load(move |_| {
        let intersection_callback = Closure::<dyn Fn(Vec<IntersectionObserverEntry>)>::new(
            move |entries: Vec<IntersectionObserverEntry>| {
                log::debug!("finding intersections");
                for entry in entries {
                    let edge = entry.target().dyn_into::<HtmlElement>().unwrap();
                    // "top" or "bottom"
                    let side = edge.dataset().get("edge").unwrap();

                    // get which shadow to show.
                    // need to get the stored element here so dyn_ref lasts
                    // for long enough
                    let top_shadow = top_shadow.get_untracked().unwrap();
                    let bottom_shadow = bottom_shadow.get_untracked().unwrap();
                    let shadow = if &side == "top" {
                        top_shadow.dyn_ref::<HtmlElement>().unwrap()
                    } else {
                        bottom_shadow.dyn_ref::<HtmlElement>().unwrap()
                    };

                    if entry.is_intersecting() {
                        edge.dataset().set("intersecting", "true").unwrap();
                        shadow.style().set_property("opacity", "0").unwrap();
                    } else {
                        edge.dataset().set("intersecting", "false").unwrap();
                        shadow.style().set_property("opacity", "1").unwrap();
                    }
                }
            },
        )
        .into_js_value()
        .unchecked_into::<Function>();

        let mut options = IntersectionObserverInit::new();
        options.root(Some(
            intersection_root
                .get_untracked()
                .unwrap()
                .dyn_ref()
                .unwrap(),
        ));

        let observer =
            IntersectionObserver::new_with_options(&intersection_callback, &options).unwrap();

        observer.observe(top_edge.get_untracked().unwrap().dyn_ref().unwrap());
        observer.observe(bottom_edge.get_untracked().unwrap().dyn_ref().unwrap());
    });

    view! {
        div class="page" {
            div class="context" ref={intersection_root} {
                div class="scroller" {
                    main ref={main_ref} {
                        div class="intersection-edge" ref={top_edge} data-edge="top";
                        HomePage;
                        div class="intersection-edge" ref={bottom_edge} data-edge="bottom";
                    }
                }
                div class="scroll-shadow" data-edge="top" ref={top_shadow};
                div class="scroll-shadow" data-edge="bottom" ref={bottom_shadow};
            }
            nav {
                button class="add mix-btn-colored-green" on:click={move |_| timers.push_new()} {
                    Icon icon="ph:plus-bold";
                }
                button class="remove mix-btn-colored-red" on:click={move |_| timers.clear()} {
                    Icon icon="ph:trash-bold";
                }
            }
        }
    }
}

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
fn retrieve_timers() -> Option<TimerList> {
    let local_storage = window().local_storage().ok()??;
    let timers_string = local_storage.get_item("timers").ok()??;
    serialize::parse_timer_json(&timers_string)
}
