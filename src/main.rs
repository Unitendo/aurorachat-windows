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

    // login, join, history
    let _ = stream.write_all(b"login|goober|skibidi|\njoin|general|\nhistory|\n");
    
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
    
    let mut message: String = String::new();
    let options = eframe::NativeOptions::default();
    eframe::run_ui_native("AuroraChat Windows", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("#general");
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
                    println!("pressed enter");
                    // send message
                    let _ = stream.write_all(&format!("msg|{}", message).into_bytes());
                    message = "".to_string();
                }
            });
        });
    })
}