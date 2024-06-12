use leptos::*;
use leptos_mview::mview;
use std::time::Duration as StdDuration;
use time::Duration;

const DURATIONS: [LabelledDuration; 8] = [
    LabelledDuration::new(Duration::days(1), "1d"),
    LabelledDuration::new(Duration::hours(1), "1h"),
    LabelledDuration::new(Duration::minutes(30), "30m"),
    LabelledDuration::new(Duration::minutes(10), "10m"),
    LabelledDuration::new(Duration::minutes(5), "5m"),
    LabelledDuration::new(Duration::minutes(1), "1m"),
    LabelledDuration::new(Duration::seconds(30), "30s"),
    LabelledDuration::new(Duration::seconds(10), "10s"),
];

#[component]
pub fn DurationUpdateButton<F>(
    on_click: F,
    #[prop(optional)] button_class: &'static str,
    #[prop(optional)] add_only: bool,
) -> impl IntoView
where
    F: Fn(Duration) + Copy + 'static,
{
    let menu_expanded = create_rw_signal(false);
    let selected_duration = create_rw_signal(DURATIONS[5]);

    let show_duration_menu = move |ev: ev::MouseEvent| {
        // needed to stop the window event listener from hiding immediately
        ev.stop_immediate_propagation();
        menu_expanded.set(true);
    };

    let select_duration = move |duration: LabelledDuration| {
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
                {DURATIONS.map(|d| {
                    mview! {
                        button
                            class={button_class}
                            on:click={move |_| select_duration(d)}
                        {
                            {d.label}
                        }
                    }
                }).collect_view()}
            }
        }
    };

    let update_button = move || {
        mview! {
            div.com-duration-button class:{add-only} {
                button 
                    class={button_class}
                    on:click={move |_| on_click(selected_duration().duration)}
                ("+")

                button
                    class={button_class}
                    on:click={show_duration_menu} 
                ( {selected_duration().label} )

                Show when=[!add_only] {
                    button 
                        class={button_class}
                        on:click={move |_| on_click(-1 * selected_duration().duration)}
                    ("-")
                }
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

#[derive(Clone, Copy)]
struct LabelledDuration {
    duration: Duration,
    label: &'static str,
}

impl LabelledDuration {
    pub const fn new(duration: Duration, label: &'static str) -> Self {
        Self {
            duration,
            label,
        }
    }
}
