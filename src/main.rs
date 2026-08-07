use eframe::egui;
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

fn on_data(data: &[u8], messages: &str) -> String {
    let msg = String::from_utf8_lossy(data);
    let mut returnstr: String = messages.to_owned();
    for line in msg.split("\n").collect::<Vec<_>>() {
        if line.starts_with("msg") {
            let formatted_msg = format!("<{}>: {}", line.split('|').collect::<Vec<_>>()[1], line.split('|').collect::<Vec<_>>()[2]);
            returnstr = returnstr + "\n" + &formatted_msg;
        }
    }
    return returnstr;
}

fn main() -> eframe::Result {
    println!("Welcome to AuroraChat Windows");
    let mut stream = TcpStream::connect("104.236.25.60:7070").expect("Failed to connect...");
    println!("Connected!");
    let mut buffer = [0u8; 2048];

    // hello
    stream.read(&mut buffer).expect("Failed to read. Not sure why this happens. Not critical.");
    
    // spawn thread for reading
    let mut stream_clone = stream.try_clone().expect("Failed to clone stream.");
    let messages = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let messages_thread = messages.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            match stream_clone.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let new_msg = on_data(&buf[..n], &messages_thread.lock().unwrap());
                    *messages_thread.lock().unwrap() = new_msg;
                }
                Err(_) => break,
            }
        }
    });
    
    let mut username: String = String::new();
    let mut password: String = String::new();
    let mut message: String = String::new();
    let mut channel: String = "general".to_string();
    let mut screen: u8 = 0;
    let options = eframe::NativeOptions::default();
    eframe::run_ui_native("AuroraChat Windows", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            if screen == 0 {
                ui.heading("login/signup");
                ui.add(egui::TextEdit::singleline(&mut username).desired_width(256.0).hint_text("Username"));
                ui.add(egui::TextEdit::singleline(&mut password).desired_width(256.0).hint_text("Password").password(true));
                ui.horizontal(|ui| {
                    if ui.button("Log In").clicked() {
                        let _ = stream.write_all(&format!("login|{}|{}|\njoin|general|\nhistory|\n", username, password).into_bytes());
                        screen = 1
                    }
                    if ui.button("Sign Up").clicked() {
                        let _ = stream.write_all(&format!("register|{}|{}|\njoin|general|\nhistory|\n", username, password).into_bytes());
                        screen = 1
                    }
                });
            } else if screen == 1 {
                ui.horizontal(|ui| {
                    ui.heading("#");
                    let textedit = ui.add(egui::TextEdit::singleline(&mut channel).desired_width(256.0));
                    if textedit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && channel != "" {
                        // change channel
                        let _ = stream.write_all(b"part|\n");
                        *messages.lock().unwrap() = String::new();
                        let _ = stream.write_all(&format!("join|{}|\nhistory|", channel).into_bytes());
                    }
                });
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .scroll_bar_visibility(ScrollBarVisibility::default())
                    .show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                            |ui| {
                                ui.colored_label(egui::Color32::WHITE, format!("{}", messages.lock().unwrap()));
                            },
                        );
                    });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    let textedit = ui.add(egui::TextEdit::singleline(&mut message).desired_width(f32::INFINITY));
                    if textedit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && message != "" {
                        // send message
                        let _ = stream.write_all(&format!("msg|{}", message).into_bytes());
                        message = "".to_string();
                    }
                });
            }
        });
    })
}