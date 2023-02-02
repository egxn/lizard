use gtk::{self, traits::{MenuShellExt, GtkMenuItemExt, WidgetExt, ContainerExt }, Label};
use image::load_from_memory;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use notify_rust::Notification;
use rust_embed::RustEmbed;
use serde_derive::{Serialize, Deserialize};
use std::{env, fs, path::PathBuf};
use std::process::Command;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct MyConfig {
    message: String,
    minutes: u64,
    title: String,
}

impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            message: "Time to take a break!".to_string(),
            minutes: 20,
            title: "Eyes break 🧐".to_string(),
        }
    }
}

fn get_icon_path() -> (PathBuf, PathBuf) {
    let dir = env::temp_dir().join("lizard");
    let icon_path = dir.join("icon.png");
    (dir, icon_path)
}

fn copy_assets() -> () {
    let (dir, icon_path) = get_icon_path();
    if !dir.exists() {
        fs::create_dir(&dir).unwrap();
    }

    load_from_memory(&Asset::get("icon.png").unwrap().data)
        .unwrap()
        .save(&icon_path)
        .unwrap();
}

fn minutes_item(time: u64, icon: char) -> gtk::MenuItem {  
    let minutes_box_item = gtk::MenuItem::new();
    let time_label = Label::new(Some(&format!("{} {} minutes", icon, time)));

    minutes_box_item.add(&time_label);
    minutes_box_item.connect_activate(move |_| {
        confy::store("lizard", None, MyConfig { 
            minutes: time,
            ..Default::default()
        }).unwrap();
    });

    minutes_box_item
}

fn exit_item() -> gtk::MenuItem {
    let item = gtk::MenuItem::with_label("🚪 Close");
    item.connect_activate(|_| gtk::main_quit());
    item
}

fn turn_on_night_light() -> gtk::MenuItem {
    let item = gtk::MenuItem::with_label("😎 Filter Light");
    item.connect_activate(|_| {
        Command::new("gsettings")
            .args(&["set", "org.gnome.settings-daemon.plugins.color", "night-light-enabled", "true"])
            .output()
            .expect("failed to execute process");
    });

    item
}

fn turn_off_night_light() -> gtk::MenuItem {
    let item = gtk::MenuItem::with_label("🤓 Normal Light");
    item.connect_activate(|_| {        
        Command::new("gsettings")
            .args(&["set", "org.gnome.settings-daemon.plugins.color", "night-light-enabled", "false"])
            .output()
            .expect("failed to execute process");
    });

    item
}

fn create_tray_icon() -> () {
    let (dir, _) = get_icon_path();
    copy_assets();
    let exit = exit_item();
    let mut menu = gtk::Menu::new();
    menu.append(&minutes_item(15, '⌛'));
    menu.append(&minutes_item(20, '⌛'));
    menu.append(&minutes_item(30, '⏳'));
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&turn_on_night_light());
    menu.append(&turn_off_night_light());
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&exit);
    menu.show_all();

    let mut tray: AppIndicator = AppIndicator::new("lizard", "🧐");
    tray.set_status(AppIndicatorStatus::Active);    
    tray.set_icon_theme_path(dir.to_str().unwrap());
    tray.set_icon_full("icon", "icon");
    tray.set_menu(&mut menu);
}

fn main() -> Result<(), confy::ConfyError> {
    let cfg: MyConfig = confy::load("lizard", None)?;    

    gtk::init().unwrap();
    create_tray_icon();
    let (_, icon_path) = get_icon_path();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(&cfg.minutes * 60));
            let notification = Notification::new()
                .summary(&cfg.title)
                .body(&cfg.message)
                .icon(&icon_path.to_str().unwrap())
                .urgency(notify_rust::Urgency::Critical)
                .show()
                .unwrap();
            std::thread::sleep(std::time::Duration::from_secs(20));
            notification.close();
        }
    });

    gtk::main();
    Ok(())
}
