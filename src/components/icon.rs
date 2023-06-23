use leptos::*;

const BASE_URL: &str = "https://api.iconify.design";

#[component]
pub fn Icon(cx: Scope, icon: &'static str, #[prop(default = false)] inline: bool) -> impl IntoView {
    let icon_svg: Resource<(), Option<String>> = create_local_resource(
        cx,
        || (),
        move |_| async move {
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
        },
    );

    move || match icon_svg.read(cx) {
        None => view! { cx,
            <span class="com-icon" data-inline=inline.to_string() />
        },
        Some(data) => view! { cx,
            <span
                class="com-icon"
                data-inline=inline.to_string()
                inner_html=data.unwrap_or_default()
            />
        },
    }
}
