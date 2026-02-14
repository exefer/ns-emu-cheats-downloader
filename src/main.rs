#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod cheat_provider;
mod curl_helper;
mod tinfoil;
mod ui;
mod utils;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use curl::easy::Easy;
use curl_helper::BodyExt;
use eframe::egui;
use tinfoil::{TinfoilRoot, TinfoilTitle};
use ui::{CheatManager, DatabaseExplorer};

#[derive(PartialEq)]
enum Tab {
    CheatManager,
    DatabaseExplorer,
}

struct App {
    active_tab: Tab,
    cheat_manager: CheatManager,
    database_explorer: DatabaseExplorer,
    folder: Option<PathBuf>,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let titles = Arc::new(Mutex::new(Vec::new()));
        Self::fetch_titles(Arc::clone(&titles));

        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            active_tab: Tab::DatabaseExplorer,
            cheat_manager: CheatManager::default(),
            database_explorer: DatabaseExplorer::new(titles),
            folder: None,
        }
    }

    fn fetch_titles(titles: Arc<Mutex<Vec<TinfoilTitle>>>) {
        thread::spawn(move || {
            let mut easy = Easy::new();
            easy.url("https://tinfoil.io/Title/ApiJson/").unwrap();
            easy.useragent(env!("CARGO_PKG_NAME")).unwrap();
            *titles.lock().unwrap() = easy
                .without_body()
                .send_with_response::<TinfoilRoot>()
                .unwrap()
                .data;
        });
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Set Mod Data Location").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_folder()
                    {
                        self.folder = Some(path);
                    }
                });
            });
        });
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                Tab::DatabaseExplorer,
                "Database Explorer",
            );
            ui.selectable_value(&mut self.active_tab, Tab::CheatManager, "Cheat Manager");
        });
    }

    fn show_content(&mut self, ui: &mut egui::Ui) {
        match self.active_tab {
            Tab::CheatManager => {
                self.cheat_manager.show(ui, self.folder.as_ref());
            }
            Tab::DatabaseExplorer => {
                if let Some(title) = self.database_explorer.show(ui) {
                    self.cheat_manager.set_title(title);
                    self.active_tab = Tab::CheatManager;
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_menu_bar(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_tabs(ui);
            self.show_content(ui);
        });
    }
}

fn main() {
    let _ = eframe::run_native(
        concat!(env!("CARGO_PKG_NAME"), " | v", env!("CARGO_PKG_VERSION"),),
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_resizable(true)
                .with_inner_size([800.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
}
