use leptos::*;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

const WEEKDAY_FORMAT: &[FormatItem<'_>] = format_description!("[weekday repr:short]");
const FULL_DATE_FORMAT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");
const TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[hour repr:12 padding:none]:[minute] [period case:lower]");

#[component]
pub fn RelativeTime(
    cx: Scope,
    #[prop(into)] time: MaybeSignal<Option<OffsetDateTime>>,
) -> impl IntoView {
    let string = create_memo(cx, move |_| {
        if time().is_none() {
            return String::new();
        };
        let time = time().unwrap();

        let current_day = OffsetDateTime::now_local().unwrap().date();
        let target_day = time.date();
        let days_between = (target_day - current_day).whole_days();

        let display_date = if days_between == 0 {
            String::new()
        } else if days_between == 1 {
            "tmr".to_string()
        } else if days_between < 7 {
            // 3 letter weekday name
            time.format(WEEKDAY_FORMAT).unwrap()
        } else {
            // yyyy-mm-dd format
            target_day.format(FULL_DATE_FORMAT).unwrap()
        };

        let end_time = time.format(TIME_FORMAT).unwrap();

        format!("{} {}", display_date, end_time).trim().to_string()
    });

    view! { cx,
        <span class="com-relative-time">
            {string}
        </span>
    }
}
