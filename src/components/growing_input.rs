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
    let size_ref = NodeRef::new();

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

#[allow(clippy::needless_pass_by_value)]
fn resize(input: HtmlElement<html::AnyElement>, size_ref: HtmlElement<html::Span>) {
    let input = input.dyn_ref::<HtmlInputElement>().unwrap().clone();
    resize_to_fit(&input, &size_ref);
    _ = use_event_listener(input.clone(), ev::keydown, {
        let input = input.clone();
        move |ev| {
            if ev.code() == "Enter" || ev.code() == "Escape" {
                input.blur().unwrap();
            }
        }
    });
    _ = use_event_listener(input.clone(), ev::input, {
        let (input, size_ref) = (input.clone(), size_ref.clone());
        move |_| resize_to_fit(&input, &size_ref)
    });
    _ = size_ref.on_mount(move |size_ref| resize_to_fit(&input, &size_ref));
}

fn resize_to_fit(input: &HtmlInputElement, size_ref: &HtmlElement<html::Span>) {
    set_size_ref(input, size_ref);
    set_input_size(input, size_ref);
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
        .get_computed_style(input)
        .unwrap()
        .unwrap()
        .get_property_value("font-size")
        .unwrap();
    // if this component is not mounted, font size will be nothing
    if font_size.is_empty() {
        return;
    };
    // remove last 2 characters "px"
    let font_size = font_size[0..font_size.len() - 2]
        .parse::<f64>()
        .expect("computed style should be valid float");
    let ems = width / font_size;
    _ = input.set_attribute("style", &format!("width: {ems}em;"));
}
