use gtk::{self, traits::{MenuShellExt, GtkMenuItemExt, WidgetExt}};
use image::load_from_memory;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use notify_rust::Notification;
use rust_embed::{RustEmbed};
use serde_derive::{Serialize, Deserialize};
use std::{env, fs};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

#[derive(Serialize, Deserialize)]
struct MyConfig {
    minutes: u64,
}

impl ::std::default::Default for MyConfig {
    fn default() -> Self { Self { minutes: 20 } }
}

fn main() -> Result<(), confy::ConfyError> {
    // confy::store("lizard", None, MyConfig { minutes: 1 })?;
    let cfg: MyConfig = confy::load("lizard", None)?;    

    gtk::init().unwrap();

    let exit = gtk::MenuItem::with_label("Exit");
    exit.connect_activate(move |_| { gtk::main_quit(); });

    let mut menu = gtk::Menu::new();
    menu.append(&exit);
    menu.show_all();

    let mut tray: AppIndicator = AppIndicator::new("lizard", "🧐");
    tray.set_status(AppIndicatorStatus::Active);

    let dir = env::temp_dir()
        .join("lizard");

    if !dir.exists() {
        fs::create_dir(&dir).unwrap();
    }

    let icon_path = dir.join("icon.png");
    load_from_memory(&Asset::get("icon.png").unwrap().data)
        .unwrap()
        .save(&icon_path)
        .unwrap();

    tray.set_icon_theme_path(dir.to_str().unwrap());
    tray.set_icon_full("icon", "icon");
    tray.set_menu(&mut menu);

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(cfg.minutes * 60));
            Notification::new()
                .summary("Eyes break 🧐")
                .body("look at something 6m away for at least 20 seconds")
                .icon(&icon_path.to_str().unwrap())
                .urgency(notify_rust::Urgency::Critical)
                .show()
                .unwrap()
                .wait_for_action(|action| match action {
                    "__closed" => { println!("Notification closed"); },
                    _ => ()
                });
        }
    });


    gtk::main();
    Ok(())
}
