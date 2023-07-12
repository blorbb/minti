use std::time::Duration;

use leptos::*;

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
    // setting the size in terms of ems so that if the font size ever changes,
    // (e.g. window resize, as font size is based on vw) it will still be the
    // correct width.
    let width = size_ref.get_bounding_client_rect().width();
    // as a string like "123px"
    let font_size = window()
        .get_computed_style(&input)
        .unwrap()
        .unwrap()
        .get_property_value("font-size")
        .unwrap();
    // remove last 2 characters "px"
    let font_size = font_size[0..font_size.len() - 2].parse::<f64>().unwrap();
    let ems = width / font_size;
    input.style("width", format!("{ems}em"));
}
