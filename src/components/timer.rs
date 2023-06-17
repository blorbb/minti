use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Local};
use leptos::*;

use crate::{
    components::duration::DurationDisplay,
    utils::{parse::parse_input, timer::Timer},
};

#[component]
pub fn TimerDisplay(cx: Scope, timer: Timer) -> impl IntoView {
    let time_remaining = timer.time_remaining;
    let update_time_remaining = move || timer.update_time_remaining();
    create_effect(cx, move |_| log!("updated! {:?}", time_remaining()));

    set_interval(update_time_remaining, Duration::from_millis(200));

    let (error_message, set_error_message) = create_signal(cx, None::<String>);
    let (end_time, set_end_time) = create_signal(cx, None::<DateTime<Local>>);

    let set_timer_duration = move |input: String| {
        let res = parse_input(&input);

        match res {
            Ok(duration) => {
                timer.reset_with_duration(duration);
                timer.start();
                set_error_message(None);

                let now = Local::now();
                let time_to_end = ChronoDuration::from_std(duration).unwrap();
                set_end_time(Some(now + time_to_end));

                log!("inside update {:?}", (timer.duration).get_untracked());
            }
            Err(e) => {
                set_error_message(Some(e.to_string()));
            }
        }
    };

    // references
    // https://stackoverflow.com/a/38867270
    let title_input_ref = create_node_ref::<html::Input>(cx);
    let size_ref = create_node_ref(cx);

    // input is after the size ref in DOM so wait for that
    // to load. If rearranged, make sure to change this too.
    title_input_ref.on_load(cx, move |elem| {
        input_resize::resize_to_fit_with_timeout(elem, size_ref().unwrap());
    });

    view! { cx,
        <div class="com-timer">
            <div class="heading">
                <span class="title">
                    <span class="size-reference" ref=size_ref></span>
                    <input
                        type="text"
                        placeholder="Enter a title"
                        ref=title_input_ref
                        on:input=move |_| input_resize::resize_to_fit(
                            title_input_ref().unwrap(),
                            size_ref().unwrap()
                        )
                    />
                </span>
                <Show
                    when=move || error_message().is_some()
                    fallback=|_| ()
                >
                    " | " <span class="error">{error_message}</span>
                </Show>
                <Show
                    when=move || end_time().is_some()
                    fallback=|_| ()
                >
                    " | " <span class="end">{move || format!("{:?}", end_time())}</span>
                </Show>
            </div>
            <div class="duration">
                <Show
                    when=move || {
                        (timer.started)()
                    }
                    fallback=move |cx| view! { cx,
                        <input
                            type="text"
                            on:keydown=move |ev| {
                                // log!("key {}", ev.key());
                                if ev.key() == "Enter" {
                                    let value = event_target_value(&ev);
                                    set_timer_duration(value);
                                };
                            }
                        />
                    }
                >
                    <DurationDisplay duration={time_remaining} />
                </Show>
            </div>
        </div>
    }
}

mod input_resize {
    use leptos::*;
    use std::time::Duration;

    pub fn resize_to_fit(input: HtmlElement<html::Input>, size_ref: HtmlElement<html::Span>) {
        set_size_ref(&input, &size_ref);
        set_input_size(input, size_ref);
    }

    pub fn resize_to_fit_with_timeout(
        input: HtmlElement<html::Input>,
        size_ref: HtmlElement<html::Span>,
    ) {
        set_size_ref(&input, &size_ref);

        // need to wait for DOM to update
        // for some reason this isn't needed on the input event
        // but is on element load
        set_timeout(move || set_input_size(input, size_ref), Duration::ZERO);
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

    fn set_input_size(input: HtmlElement<html::Input>, size_ref: HtmlElement<html::Span>) {
        let width = size_ref.offset_width();
        input.style("width", format!("{width}px"));
    }
}
