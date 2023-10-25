use std::time::Duration;

use leptos::*;
use leptos_mview::mview;

/// An input element that grows with the input size.
#[component]
pub fn GrowingInput<F>(
    placeholder: &'static str,
    /// Functions cannot be optional, so one must be passed in.
    ///
    /// If you don't want to do anything, set this to `|_| ()`.
    on_input: F,
    #[prop(optional)] initial: String,
) -> impl IntoView
where
    F: Fn(ev::Event) + 'static,
{
    // references
    // https://stackoverflow.com/a/38867270
    let title_input_ref = create_node_ref::<html::Input>();
    let size_ref = create_node_ref();

    // input is after the size ref in DOM so wait for that
    // to load. If rearranged, make sure to change this too.
    title_input_ref.on_load(move |elem| {
        // need to wait slightly for initial value to be set.
        // set_input_size also needs to be delayed here even without an
        // initial value.
        set_timeout(
            move || {
                resize_to_fit(
                    elem,
                    &size_ref
                        .get_untracked()
                        .expect("`size_ref` should be loaded before `title_input_ref`"),
                );
            },
            Duration::ZERO,
        );
    });

    let on_keydown = move |ev: ev::KeyboardEvent| {
        if ev.code() == "Enter" || ev.code() == "Escape" {
            title_input_ref.get_untracked().unwrap().blur().unwrap();
        };
    };

    mview! {
        span.com-growing-input {
            span.size-reference ref={size_ref};
            input
                type="text"
                {placeholder}
                ref={title_input_ref}
                value={initial}
                on:input={move |ev| {
                    resize_to_fit(title_input_ref().unwrap(), &size_ref().unwrap());
                    on_input(ev)
                }}
                on:keydown={on_keydown};
        }
    }
}

fn resize_to_fit(input: HtmlElement<html::Input>, size_ref: &HtmlElement<html::Span>) {
    set_size_ref(&input, size_ref);
    set_input_size(input, size_ref);
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
    let font_size = font_size[0..font_size.len() - 2]
        .parse::<f64>()
        .expect("computed style should be valid float");
    let ems = width / font_size;
    input.style("width", format!("{ems}em"));
}
