use leptos::*;
use leptos_mview::mview;

use crate::{components::TimerDisplay, contexts::TimerList, timer::Timer};

#[component]
pub fn HomePage() -> impl IntoView {
    let timers = expect_context::<TimerList>();

    mview! {
        div.page-home {
            For
                each={timers.vec_signal()}
                key={Timer::id}
            |timer| { TimerDisplay {timer}; }
        }
    }
}
