// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::{menu::*, App, Manager, State, Wry};

#[tauri::command]
fn alert_window(window: tauri::Window) {
    window
        .request_user_attention(Some(tauri::UserAttentionType::Critical))
        .expect("should be able to request user attention");
}

#[tauri::command]
fn contextmenu(window: tauri::Window, ctx: State<GlobalContextMenu>) {
    ctx.0.popup(window).unwrap();
}

#[tauri::command]
fn set_contextmenu_checkitem(ctx: State<GlobalContextMenu>, path: String, checked: bool) {
    println!("received {path} {checked}");
    let menu_item =
        get_nested_menu_item(ctx.inner(), &path).expect("invalid menu item path provided");
    menu_item
        .as_check_menuitem_unchecked()
        .set_checked(checked)
        .unwrap();
}

struct GlobalContextMenu(Menu<Wry>);

struct RadioSubmenu {
    submenu: Submenu<Wry>,
    items: Vec<CheckMenuItem<Wry>>,
}

impl RadioSubmenu {
    /// The ids in `items` will be auto-prefixed with `submenu_id`.
    pub fn new(
        app: &App,
        submenu_id: impl Into<MenuId>,
        text: impl AsRef<str>,
        items: Vec<(&str, &str)>,
    ) -> tauri::Result<Arc<Self>> {
        let submenu_id = submenu_id.into().0;
        let submenu = SubmenuBuilder::with_id(app, submenu_id.clone(), text).build()?;
        let mut check_items = Vec::new();

        for (check_id, text) in items {
            let check_id = format!("{}::{}", &submenu_id, check_id);
            let check_item = CheckMenuItemBuilder::with_id(&check_id, text)
                .checked(false)
                .build(app)?;

            submenu.append(&check_item)?;
            check_items.push(check_item);
        }

        let this = Arc::new(Self {
            submenu,
            items: check_items,
        });

        let this2 = Arc::clone(&this);
        app.on_menu_event(move |app, ev| {
            if this2.select(ev.id()) {
                app.emit(
                    &format!("contextmenu::{}", submenu_id),
                    ev.id.0.strip_prefix(&format!("{submenu_id}::")).unwrap(),
                )
                .unwrap();
            }
        });
        Ok(this)
    }

    /// The id should be the prefixed id.
    ///
    /// Returns whether any item was actually selected.
    pub fn select(&self, id: impl AsRef<str>) -> bool {
        let id = id.as_ref();

        // make sure one of them has the provided id
        if self.items.iter().all(|item| item.id() != id) {
            return false;
        }

        for item in &self.items {
            item.set_checked(item.id() == &id).unwrap();
        }

        true
    }
}

fn get_nested_menu_item(menu: &GlobalContextMenu, path: &str) -> Option<MenuItemKind<Wry>> {
    if !path.contains("::") {
        return menu.0.get(path);
    };

    // turn something like a::b::c into [a::b, a::b::c]
    // does not include the first segment "a"
    // the first segment will be stored in `curr_prefix` at the end
    // of the loop
    let mut path_prefixes = vec![];
    let mut curr_prefix = path;
    // insert longest to shortest
    while let Some(index) = curr_prefix.rfind("::") {
        path_prefixes.push(&path[..index + 2]);
        curr_prefix = &curr_prefix[..index];
    }
    // reverse to put in order
    path_prefixes.reverse();

    let mut curr_submenu = menu.0.get(curr_prefix)?.as_submenu()?.clone();
    // all except the last path
    for prefix in path_prefixes.iter().take(path_prefixes.len() - 1) {
        curr_submenu = curr_submenu.get(*prefix)?.as_submenu()?.clone();
    }

    curr_submenu.get(path)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let radio_menu = RadioSubmenu::new(
                app,
                "timer-face",
                "Timer face appearance",
                vec![
                    ("opaque", "Opaque"),
                    ("transparent", "Transparent"),
                    ("blur", "Blur"),
                ],
            )?;

            let menu = MenuBuilder::new(app)
                .text("add-timer", "Add timer")
                .text("delete-all", "Delete all timers")
                .separator()
                .item(&radio_menu.submenu)
                .item(
                    &SubmenuBuilder::with_id(app, "heading-show", "Show in heading")
                        .check("heading-show::title", "Title")
                        .check("heading-show::end-time", "End time")
                        .check("heading-show::elapsed", "Elapsed time")
                        .build()?,
                )
                .build()?;

            app.manage(GlobalContextMenu(menu));

            app.on_menu_event(|app, event| {
                println!("received event {event:?}");
                let menu = app.state::<GlobalContextMenu>().inner();

                if event.id() == "add-timer" {
                    app.emit("contextmenu::add-timer", String::from("added timer"))
                        .unwrap();
                    println!("adding timer");
                } else if event.id() == "delete-all" {
                    app.emit(
                        "contextmenu::delete-all",
                        String::from("deleted all timers"),
                    )
                    .unwrap();
                    println!("deleting all timers");
                } else if let Some(option) = event.id().0.strip_prefix("heading-show::") {
                    let menu_item = get_nested_menu_item(menu, event.id().0.as_str()).unwrap();
                    let menu_item = menu_item.as_check_menuitem_unchecked();

                    println!(
                        "emitting contextmenu::heading-show with {option}={:?}",
                        menu_item.is_checked()
                    );
                    app.emit(
                        "contextmenu::heading-show",
                        format!("{}={}", option, menu_item.is_checked().unwrap()),
                    )
                    .unwrap();
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            alert_window,
            contextmenu,
            set_contextmenu_checkitem
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
