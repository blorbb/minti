use leptos::*;

use crate::{components::Icon, utils::contexts::FullscreenElement};

#[component]
pub fn FullscreenButton(
    cx: Scope,
    target: NodeRef<html::Div>,
    class: &'static str,
) -> impl IntoView {
    let fullscreen_element = expect_context::<FullscreenElement>(cx);
    let is_fullscreen = create_memo(cx, move |_| {
        let result = fullscreen_element().is_some();
        log!("toggling to {}", result);
        result
    });

    let enable_fullscreen = move |_| {
        let Some(elem) = target() else { return };
        _ = elem.request_fullscreen();
    };

    let disable_fullscreen = move |_| {
        if !document().fullscreen_enabled() || fullscreen_element.get_untracked().is_none() {
            return;
        };
        document().exit_fullscreen();
    };

    view! { cx,
        <Show
            when=is_fullscreen
            fallback=move |cx| {
                view! { cx,
                    <button class=format!("com-fullscreen-button {}", class) on:click=enable_fullscreen>
                        <Icon icon="ph:corners-out"/>
                    </button>
                }
            }
        >
            <button class=format!("com-fullscreen-button {}", class) on:click=disable_fullscreen>
                <Icon icon="ph:corners-in"/>
            </button>
        </Show>
    }
}
