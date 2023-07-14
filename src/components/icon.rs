use leptos::*;

const BASE_URL: &str = "https://api.iconify.design";

/// Displays an svg icon.
///
/// Icons are from `iconify-icon` and are expected to be valid.
#[component]
pub fn Icon(
    cx: Scope,
    /// The `iconify-icon` icon id. See <https://icon-sets.iconify.design/>.
    #[prop(into)]
    icon: MaybeSignal<&'static str>,
) -> impl IntoView {
    // warnings are probably fixed in https://github.com/leptos-rs/leptos/pull/1342

    let icon_svg: Resource<_, Option<String>> =
        create_local_resource(cx, icon, move |icon| async move {
            if let Some(body) = window()
                .local_storage()
                .unwrap()
                .unwrap()
                .get_item(&format!("icon.{}", icon))
                .unwrap()
            {
                // TODO: probably should sanitise the body just in case
                log::debug!("found icon {} in localstorage", icon);
                return Some(body);
            };

            log::debug!("fetching icon {}", icon);
            let (prefix, name) = icon.split_once(':')?;

            let body = reqwest::get(format!("{}/{}/{}.svg", BASE_URL, prefix, name))
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
                Some(body)
            }
        });

    view! { cx,
        <span
            class="com-icon"
            inner_html=move || icon_svg.read(cx).flatten().unwrap_or_default()
        />
    }
}
