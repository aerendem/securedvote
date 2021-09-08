use eframe::{egui, epi};

use async_std::task;
use crate::ballotchain::{self, Ballotchain};
use crate::ballot::Ballot;
use crate::now;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct SecVApp {
    // Example stuff:
    label: String,
    value: f32,
}


static mut last_hash: Vec<u8> = vec![0; 32];

pub fn change_last_hash(new_last_hash: Vec<u8>) {
    unsafe {
        last_hash = new_last_hash.clone();
    }
    
}

impl Default for SecVApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            
        }
    }
}

impl SecVApp {
    pub fn change_last_hash(new_last_hash: Vec<u8>) {
        unsafe {
            last_hash = new_last_hash.clone();
        }
        
    }
}
impl epi::App for SecVApp {
    fn name(&self) -> &str {
        "Secured Vote"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let SecVApp {
            label,
            value,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Candidates");

            
            ui.horizontal(|ui| {
                ui.label("Adam Smith");
                ui.label("Candidate Id: 001");
            });

            
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });
        

        egui::CentralPanel::default().show(ctx, |ui| { 
            ui.heading("Voting Panel");
            ui.separator();
            
        
            
            ui.vertical_centered(|ui|{
                ui.label("Enter Candidate ID to vote ");
                let mut my_string = String::from("");
                let response = ui.add(egui::TextEdit::singleline(&mut my_string));
                if response.changed() {
                    // …
                }
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    // …
                }
                if ui.add(egui::Button::new("Vote")).clicked() {
                    let difficulty = 0x000fffffffffffffffffffffffffffff;
                    let mut new_last_hash;
                    unsafe {
                        new_last_hash = last_hash.clone();
                        
                    }
                    let mut ballot = Ballot::new(2, now(), new_last_hash, 0, 365, difficulty);
                    ballot.vote(0);
                    println!("Voted(mined) with ballot {:?}", &ballot);
                    Ballotchain::update_with_block(&mut Ballotchain, ballot)
                }
            });
           
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/aerendem/securedvote").text("source code"),
                );
            });
            //egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}



