use leptos::*;
use leptos_mview::mview;
use leptos_use::use_event_listener;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

/// An input element that grows with the input size.
#[component]
pub fn GrowingInput(
    placeholder: &'static str,
    /// Functions cannot be optional, so one must be passed in.
    ///
    /// If you don't want to do anything, set this to `|_| ()`.
    on_input: impl Fn(ev::Event) + 'static,
    #[prop(optional)] initial: String,
) -> impl IntoView {
    // references
    // https://stackoverflow.com/a/38867270
    let size_ref = create_node_ref();

    mview! {
        span.com-growing-input {
            span.size-reference ref={size_ref};
            input
                type="text"
                {placeholder}
                value={initial}
                on:input={on_input}
                use:resize={size_ref.get_untracked().unwrap()};
        }
    }
}

fn resize(input: HtmlElement<html::AnyElement>, size_ref: HtmlElement<html::Span>) {
    let input = input.dyn_ref::<HtmlInputElement>().unwrap().clone();
    resize_to_fit(&input, &size_ref);
    _ = use_event_listener(input.clone(), ev::keydown, |ev| {
        if ev.code() == "Enter" || ev.code() == "Escape" {
            event_target::<HtmlInputElement>(&ev).blur().unwrap();
        }
    });
    _ = use_event_listener(input.clone(), ev::input, move |ev| {
        resize_to_fit(&event_target::<HtmlInputElement>(&ev), &size_ref)
    });
}

fn resize_to_fit(input: &HtmlInputElement, size_ref: &HtmlElement<html::Span>) {
    set_size_ref(input, &size_ref);
    set_input_size(input, &size_ref);
}

fn set_size_ref(input: &HtmlInputElement, size_ref: &HtmlElement<html::Span>) {
    let input_text = input.value();
    let placeholder = input.placeholder();

    if input_text.is_empty() {
        size_ref.set_text_content(Some(&placeholder));
    } else {
        size_ref.set_text_content(Some(&input_text));
    }
}

fn set_input_size(input: &HtmlInputElement, size_ref: &HtmlElement<html::Span>) {
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
    _ = input.set_attribute("style", &format!("width: {ems}em;"));
}
