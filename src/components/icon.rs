use gloo_net::http::Request;
use leptos::*;
use leptos_mview::mview;

use crate::utils::contexts::Icons;

const BASE_URL: &str = "https://api.iconify.design";

/// Displays an svg icon.
///
/// Icons are from `iconify-icon` and are expected to be valid.
#[component]
pub fn Icon(
    /// The `iconify-icon` icon id. See <https://icon-sets.iconify.design/>.
    #[prop(into)]
    icon: MaybeSignal<&'static str>,
) -> impl IntoView {
    // warnings are probably fixed in https://github.com/leptos-rs/leptos/pull/1342

    let stored_icons = expect_context::<Icons>();

    let icon_svg: Resource<_, Option<String>> =
        create_local_resource(icon, move |icon| async move {
            if let Some(body) = stored_icons.get(icon) {
                // TODO: probably should sanitise the body just in case
                log::debug!("found icon {} in localstorage", icon);
                return Some(body);
            };

            log::debug!("fetching icon {}", icon);
            let (prefix, name) = icon.split_once(':')?;

            let body = Request::get(&format!("{}/{}/{}.svg", BASE_URL, prefix, name))
                .send()
                .await
                .ok()?
                .text()
                .await
                .ok()?;

            if body == "404" {
                log::error!("icon {} not found!", icon);
                None
            } else {
                log::debug!("successfully fetched icon {}", icon);
                stored_icons.add(icon, &body);
                Some(body)
            }
        });

    mview! {
        span.com-icon inner_html=[icon_svg.get().flatten().unwrap_or_default()];
    }
}
