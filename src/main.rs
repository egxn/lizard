use gtk;
use notify_rust::Notification;
use serde_derive::{Serialize, Deserialize};
use tray_item::TrayItem;

#[derive(Serialize, Deserialize)]
struct MyConfig {
    minutes: u64,
}

impl ::std::default::Default for MyConfig {
    fn default() -> Self { Self { minutes: 20 } }
}

fn main() -> Result<(), confy::ConfyError> {
    gtk::init().unwrap();
    let cfg: MyConfig = confy::load("lizard", None)?;
    let mut tray = TrayItem::new("lizard", "🧐").unwrap();

    tray.add_label("Lizard").unwrap();
    tray.add_menu_item("Quit", || { gtk::main_quit(); }).unwrap();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(cfg.minutes * 60));
            Notification::new()
                .summary("Eyes break 🧐")
                .body("look at something 20 feet away for at least 20 seconds")
                .icon("")
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
