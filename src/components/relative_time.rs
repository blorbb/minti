use leptos::*;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

use crate::utils::time::relative;

const WEEKDAY_FORMAT: &[FormatItem<'_>] = format_description!("[weekday repr:short]");
const FULL_DATE_FORMAT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");
const TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[hour repr:12 padding:none]:[minute] [period case:lower]");

/// Displays a time (and date when needed) relative to now.
///
/// Only updates when `time` is updated.
///
/// Renders an empty span if `None` is provided.
#[component]
pub fn RelativeTime(#[prop(into)] time: MaybeSignal<Option<OffsetDateTime>>) -> impl IntoView {
    let string = create_memo(move |_| {
        // ignore `None`
        if time().is_none() {
            return String::new();
        };
        let time = time().expect("`None` should have caused early return");

        // display a date if the target is on a different day.
        let current_day = relative::now().date();
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

    view! { <span class="com-relative-time">{string}</span> }
}
