use leptos::*;
use leptos_mview::mview;

use crate::{components::Icon, contexts::FullscreenElement};

#[component]
pub fn FullscreenButton(target: NodeRef<html::Div>, class: &'static str) -> impl IntoView {
    let fullscreen_element = expect_context::<FullscreenElement>();
    let is_fullscreen = create_memo(move |_| fullscreen_element().is_some());

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

    move || {
        if is_fullscreen() {
            mview! {
                button class=f["com-fullscreen-button {class}"] on:click={disable_fullscreen} {
                    Icon icon="ph:corners-in";
                }
            }
        } else {
            mview! {
                button class=f["com-fullscreen-button {class}"] on:click={enable_fullscreen} {
                    Icon icon="ph:corners-out";
                }
            }
        }
    }
}
