use leptos::*;
use leptos_mview::mview;
use std::time::Duration as StdDuration;
use time::ext::NumericalDuration;
use time::Duration;

#[component]
pub fn DurationUpdateButton<F>(
    on_click: F,
    add: bool,
    #[prop(optional)] button_class: &'static str,
) -> impl IntoView
where
    F: Fn(Duration) + Copy + 'static,
{
    let duration_prefix = if add { "+" } else { "-" };
    let duration_multiplier = if add { 1 } else { -1 };
    let durations = store_value([
        LabelledDuration::new(1.days(), "1d"),
        LabelledDuration::new(1.hours(), "1h"),
        LabelledDuration::new(30.minutes(), "30m"),
        LabelledDuration::new(10.minutes(), "10m"),
        LabelledDuration::new(5.minutes(), "5m"),
        LabelledDuration::new(1.minutes(), "1m"),
        LabelledDuration::new(30.seconds(), "30s"),
        LabelledDuration::new(10.seconds(), "10s"),
    ]);
    let menu_expanded = create_rw_signal(false);
    let selected_duration = create_rw_signal(durations()[5].clone());

    let oncontextmenu = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        menu_expanded.set(true);
    };

    let onmenuselect = move |duration: LabelledDuration| {
        selected_duration.set(duration);
        menu_expanded.set(false);
    };

    // clicking anywhere should hide
    let click_handler = window_event_listener(ev::click, move |_| {
        if menu_expanded() {
            menu_expanded.set(false);
        };
    });
    // esc should hide as well
    let key_handler = window_event_listener(ev::keydown, move |ev| {
        if menu_expanded() && ev.key() == "Escape" {
            menu_expanded.set(false);
        };
    });

    on_cleanup(|| {
        click_handler.remove();
        key_handler.remove();
    });

    // TODO focus capturing

    let menu = move || {
        mview! {
            div.com-duration-menu {
                {durations().map(|d| {
                    let label = d.label.clone();
                    mview! {
                        button
                            class={button_class}
                            on:click={move |_| onmenuselect(d.clone())}
                        {
                            {duration_prefix} {label}
                        }
                    }
                }).collect_view()}
            }
        }
    };


    let update_button = move || {
        mview! {
            button
                class={button_class}
                on:click={move |_| on_click(duration_multiplier * selected_duration().duration)}
                on:contextmenu={oncontextmenu}
            {
                {duration_prefix} [selected_duration().label]
            }
        }
    };

    mview! {
        {update_button}
        AnimatedShow
            when={menu_expanded}
            hide-class="hiding"
            hide-delay={StdDuration::from_millis(200)}
        {
            {menu}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelledDuration {
    duration: Duration,
    label: String,
}

impl LabelledDuration {
    pub fn new(duration: Duration, label: &str) -> Self {
        Self {
            duration,
            label: label.to_string(),
        }
    }
}
