use std::time::Duration;

use leptos::*;
use wasm_bindgen::{prelude::Closure, JsCast};

/// An input element that grows with the input size.
#[component]
pub fn GrowingInput(cx: Scope, placeholder: &'static str) -> impl IntoView {
    // references
    // https://stackoverflow.com/a/38867270
    let title_input_ref = create_node_ref::<html::Input>(cx);
    let size_ref = create_node_ref(cx);

    // input is after the size ref in DOM so wait for that
    // to load. If rearranged, make sure to change this too.
    title_input_ref.on_load(cx, move |elem| {
        resize_to_fit_with_timeout(elem, size_ref().unwrap());
    });

    // font size adapts to screen size. need to update when the screen resizes.

    // can't use leptos::window_event_listener as it doesn't remove the event
    // listener after this cx is disposed, causing panic in .get_untracked().
    // .try_get_untracked() doesn't exist either.

    // window_event_listener(ev::resize, move |_| {
    //     if let Some(input) = title_input_ref.get_untracked() && let Some(size) = size_ref.get_untracked() {
    //         resize_to_fit(
    //             input,
    //             &size,
    //         )
    //     }
    // });

    let window_resize_fn = Closure::<dyn Fn()>::wrap(Box::new(move || {
        resize_to_fit(
            title_input_ref.get_untracked().unwrap(),
            &size_ref.get_untracked().unwrap(),
        );
    }))
    .into_js_value()
    .unchecked_into();

    window()
        .add_event_listener_with_callback("resize", &window_resize_fn)
        .unwrap();

    on_cleanup(cx, move || {
        window()
            .remove_event_listener_with_callback("resize", &window_resize_fn)
            .unwrap();
    });

    view! { cx,
        <span class="com-growing-input">
            <span class="size-reference" ref=size_ref></span>
            <input
                type="text"
                placeholder=placeholder
                ref=title_input_ref
                on:input=move |_| resize_to_fit(
                    title_input_ref().unwrap(),
                    &size_ref().unwrap()
                )
            />
        </span>
    }
}

fn resize_to_fit(input: HtmlElement<html::Input>, size_ref: &HtmlElement<html::Span>) {
    set_size_ref(&input, size_ref);
    set_input_size(input, size_ref);
}

fn resize_to_fit_with_timeout(input: HtmlElement<html::Input>, size_ref: HtmlElement<html::Span>) {
    set_size_ref(&input, &size_ref);

    // need to wait for DOM to update
    // for some reason this isn't needed on the input event
    // but is on element load
    set_timeout(move || set_input_size(input, &size_ref), Duration::ZERO);
}

fn set_size_ref(input: &HtmlElement<html::Input>, size_ref: &HtmlElement<html::Span>) {
    let input_text = input.value();
    let placeholder = input.placeholder();

    if input_text.is_empty() {
        size_ref.set_text_content(Some(&placeholder));
    } else {
        size_ref.set_text_content(Some(&input_text));
    }
}

fn set_input_size(input: HtmlElement<html::Input>, size_ref: &HtmlElement<html::Span>) {
    let width = size_ref.get_bounding_client_rect().width();
    input.style("width", format!("{width}px"));
}
