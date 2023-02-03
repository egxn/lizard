use gtk::prelude::*;
use gtk::{self, traits::{MenuShellExt, GtkMenuItemExt, WidgetExt, ContainerExt, DialogExt }, Label };
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

fn config_item() -> gtk::MenuItem {
    let config_item = gtk::MenuItem::with_label("🔧 Config");
    config_item.connect_activate(|_| {
        let dialog = dialog_config();
        dialog.map(|dialog| {
            dialog.show_all();
            dialog.run();
        }).expect("failed to run dialog");
    });

    config_item
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


fn dialog_config() -> Result< gtk::Dialog, confy::ConfyError> {
    let cfg: MyConfig = confy::load("lizard", None)?;
    let dialog = gtk::Dialog::new();
    dialog.set_title("Lizard Config");
    dialog.set_modal(true);
    dialog.set_default_size(300, 220);

    let box_config = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let lbl_title = gtk::Label::new(Some("Title:"));
    lbl_title.set_halign(gtk::Align::Start);
    lbl_title.set_margin_start(10);
    let entry_title = gtk::Entry::new();
    entry_title.set_halign(gtk::Align::Start);
    entry_title.set_margin_start(10);
    entry_title.set_size_request(280, 20);
    entry_title.set_text(&cfg.title);
    let lbl_message = gtk::Label::new(Some("Message:"));
    lbl_message.set_halign(gtk::Align::Start);
    lbl_message.set_margin_start(10);
    let entry_message = gtk::Entry::new();
    entry_message.set_halign(gtk::Align::Start);
    entry_message.set_margin_start(10);
    entry_message.set_size_request(280, 20);
    entry_message.set_text(&cfg.message);
    let lbl_minutes = gtk::Label::new(Some("Minutes:"));
    lbl_minutes.set_halign(gtk::Align::Start);
    lbl_minutes.set_margin_start(10);
    let entry_minutes = gtk::SpinButton::with_range(1.0, 60.0, 1.0);
    entry_minutes.set_halign(gtk::Align::End);
    entry_minutes.set_margin_start(10);
    entry_minutes.set_value(cfg.minutes as f64);

    let title = entry_title.text().to_string();
    let message = entry_message.text().to_string();
    let minutes = entry_minutes.value() as u64;

    let title_for_minutes = title.clone();
    let message_for_minutes = message.clone();

    entry_message.connect_changed(move |entry| {
        confy::store("lizard", None, MyConfig {
            title: title.clone(),
            message: entry.text().to_string(),
            minutes: minutes,
        }).unwrap();
    });

    entry_title.connect_changed(move |entry| {
        confy::store("lizard", None, MyConfig {
            title: entry.text().to_string(),
            message: message.clone(),
            minutes: minutes,
        }).unwrap();
    });

    entry_minutes.connect_value_changed(move |entry| {
        confy::store("lizard", None, MyConfig {
            title: title_for_minutes.clone(),
            message: message_for_minutes.clone(),
            minutes: entry.value() as u64,
        }).unwrap();
    });

    box_config.add(&lbl_title);
    box_config.add(&entry_title);
    box_config.add(&lbl_message);
    box_config.add(&entry_message);
    box_config.add(&lbl_minutes);
    box_config.add(&entry_minutes);

    dialog.content_area().add(&box_config);

    Ok(dialog)
}

fn create_tray_icon() -> () {
    let (dir, _) = get_icon_path();
    copy_assets();
    let exit: gtk::MenuItem = exit_item();
    let mut menu: gtk::Menu = gtk::Menu::new();
    menu.append(&turn_on_night_light());
    menu.append(&turn_off_night_light());
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&minutes_item(15 , '⏳'));
    menu.append(&minutes_item(20 , '⌛'));
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&config_item());
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
