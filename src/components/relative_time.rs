use chrono::{DateTime, Datelike, Local};
use leptos::*;

#[component]
pub fn RelativeTime(
    cx: Scope,
    #[prop(into)] time: MaybeSignal<Option<DateTime<Local>>>,
) -> impl IntoView {
    let string = create_memo(cx, move |_| {
        if time().is_none() {
            return "".to_string();
        };
        let time = time().unwrap();

        let current_time = Local::now();
        let current_day = current_time.date_naive();

        let target_day = time.date_naive();

        let days_between = (target_day - current_day).num_days();

        let display_date = if days_between == 0 {
            None
        } else if days_between == 1 {
            Some("tmr".to_string())
        } else if days_between < 7 {
            let binding = time.weekday().to_string();
            Some(binding)
        } else {
            Some(target_day.format("%F").to_string())
        };

        let end_time = time.format("%l:%M %P");

        match display_date {
            None => format!("{end_time}"),
            Some(s) => format!("{s} {end_time}"),
        }
    });

    view! { cx,
        <span class="com-relative-time">
            {string}
        </span>
    }
}
