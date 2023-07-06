use leptos::*;

const BASE_URL: &str = "https://api.iconify.design";

/// Displays an svg icon.
///
/// Icons are from `iconify-icon` and are expected to be valid.
///
/// Use the `inline` property to make the icon have the same vertical
/// alignment as surrounding text.
#[component]
pub fn Icon(
    cx: Scope,
    /// The `iconify-icon` icon id. See <https://icon-sets.iconify.design/>.
    #[prop(into)]
    icon: MaybeSignal<&'static str>,
) -> impl IntoView {
    let icon_svg: Resource<_, Option<String>> =
        create_local_resource(cx, icon, move |icon| async move {
            let (prefix, name) = icon.split_once(':')?;

            let body = reqwest::get(format!("{}/{}/{}.svg", BASE_URL, prefix, name))
                .await
                .ok()?
                .text()
                .await
                .ok()?;

            if body == "404" {
                error!("Icon {} not found!", icon);
                None
            } else {
                Some(body)
            }
        });

    move || match icon_svg.read(cx) {
        None => view! { cx,
            <span class="com-icon" />
        },
        Some(data) => view! { cx,
            <span
                class="com-icon"
                inner_html=data.unwrap_or_default()
            />
        },
    }
}
