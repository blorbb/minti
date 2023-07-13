use leptos::*;

use crate::{app::FullscreenElement, components::Icon};

#[component]
pub fn FullscreenButton(cx: Scope, target: NodeRef<html::Div>) -> impl IntoView {
    let fullscreen_element = expect_context::<ReadSignal<FullscreenElement>>(cx);
    let is_fullscreen = create_memo(cx, move |_| {
        let result = fullscreen_element().0.is_some();
        log!("toggling to {}", result);
        result
    });

    let enable_fullscreen = move |_| {
        let Some(elem) = target() else { return };
        _ = elem.request_fullscreen();
    };

    let disable_fullscreen = move |_| {
        if !document().fullscreen_enabled() || fullscreen_element.get_untracked().0.is_none() {
            return;
        };
        _ = document().exit_fullscreen();
    };

    view! { cx,
        <Show
            when=is_fullscreen
            fallback=move |cx| view! { cx,
                <button
                    class="com-fullscreen-button"
                    on:click=enable_fullscreen
                >
                    <Icon icon="ph:corners-out" />
                </button>
            }
        >
            <button
                class="com-fullscreen-button"
                on:click=disable_fullscreen
            >
                <Icon icon="ph:corners-in" />
            </button>
        </Show>

    }
}
