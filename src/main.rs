#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod cheat_provider;
mod curl_helper;
mod tinfoil;
mod utils;

use cheat_provider::CheatSource;
use curl::easy::Easy;
use curl_helper::BodyExt;
use eframe::egui;
use egui::vec2;
use egui_extras::{Column, TableBuilder};
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
    thread,
};
use tinfoil::{TinfoilRoot, TinfoilTitle};

#[derive(PartialEq)]
enum Tab {
    DatabaseExplorer,
    CheatManager,
}

struct App {
    active_tab: Tab,
    selected_cheat_source: CheatSource,
    selected_title_id: Option<String>,
    selected_title_image_bytes: Arc<Mutex<Option<Vec<u8>>>>,
    last_fetched_title_id: Option<String>,
    titles: Arc<Mutex<Vec<TinfoilTitle>>>,
    titles_filter: String,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let titles = Arc::new(Mutex::new(Vec::new()));
        Self::fetch_titles(Arc::clone(&titles));

        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            active_tab: Tab::DatabaseExplorer,
            selected_cheat_source: CheatSource::GbaTemp,
            selected_title_id: None,
            selected_title_image_bytes: Arc::new(Mutex::new(None)),
            last_fetched_title_id: None,
            titles,
            titles_filter: String::new(),
        }
    }

    fn fetch_titles(titles: Arc<Mutex<Vec<TinfoilTitle>>>) {
        thread::spawn(move || {
            let mut easy = Easy::new();
            easy.url("https://tinfoil.io/Title/ApiJson/").unwrap();
            *titles.lock().unwrap() = easy
                .without_body()
                .send_with_response::<TinfoilRoot>()
                .unwrap()
                .data;
        });
    }

    fn title_ui(&mut self, ui: &mut egui::Ui) {
        let Some(selected_id) = &self.selected_title_id else {
            ui.label("No title selected. Please go to Database Explorer and select a title.");
            return;
        };

        let titles = self.titles.lock().unwrap();
        let Some(title) = titles.iter().find(|t| &t.id == selected_id) else {
            return;
        };

        ui.horizontal(|ui| {
            ui.label("ID:");
            ui.label(&title.id);
        });
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.label(&title.name);
        });
        ui.horizontal(|ui| {
            ui.label("Release Date:");
            ui.label(title.release_date.as_deref().unwrap_or("N/A"));
        });
        ui.horizontal(|ui| {
            ui.label("Publisher:");
            ui.label(title.publisher.as_deref().unwrap_or("N/A"));
        });

        if let Some(ref image_bytes) = *self.selected_title_image_bytes.lock().unwrap() {
            ui.add(
                egui::Image::new(egui::ImageSource::Bytes {
                    uri: Cow::Owned(format!("bytes://{}.jpeg", title.id)),
                    bytes: egui::load::Bytes::Shared(Arc::from(image_bytes.as_slice())),
                })
                .max_size(vec2(200.0, 200.0)),
            );
        }

        egui::ComboBox::from_label("Cheat Source")
            .selected_text(self.selected_cheat_source.as_str())
            .show_ui(ui, |ui| {
                let s = &mut self.selected_cheat_source;
                ui.selectable_value(s, CheatSource::Blawar, CheatSource::Blawar.as_str());
                ui.selectable_value(s, CheatSource::Chansey, CheatSource::Chansey.as_str());
                ui.selectable_value(s, CheatSource::CheatSlips, CheatSource::CheatSlips.as_str());
                ui.selectable_value(s, CheatSource::GbaTemp, CheatSource::GbaTemp.as_str());
                ui.selectable_value(s, CheatSource::Hamlet, CheatSource::Hamlet.as_str());
                ui.selectable_value(s, CheatSource::Ibnux, CheatSource::Ibnux.as_str());
                ui.selectable_value(s, CheatSource::Tinfoil, CheatSource::Tinfoil.as_str());
            });
    }

    fn table_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.titles_filter);
        });

        TableBuilder::new(ui)
            .columns(Column::remainder().at_least(150.0).resizable(true), 4)
            .striped(true)
            .sense(egui::Sense::click())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("ID");
                });
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Release Date");
                });
                header.col(|ui| {
                    ui.heading("Publisher");
                });
            })
            .body(|body| {
                let titles = self.titles.lock().unwrap();
                let titles: Vec<_> = if self.titles_filter.is_empty() {
                    titles.iter().collect()
                } else {
                    titles
                        .iter()
                        .filter(|title| {
                            title.name.contains(&self.titles_filter)
                                || title.id.contains(&self.titles_filter)
                        })
                        .collect()
                };
                body.rows(18.0, titles.len(), |mut row| {
                    let Some(title) = titles.get(row.index()) else {
                        return;
                    };

                    row.col(|ui| {
                        ui.label(&title.id);
                    });
                    row.col(|ui| {
                        ui.label(&title.name);
                    });
                    row.col(|ui| {
                        ui.label(title.release_date.as_deref().unwrap_or("N/A"));
                    });
                    row.col(|ui| {
                        ui.label(title.publisher.as_deref().unwrap_or("N/A"));
                    });

                    if !row.response().clicked() {
                        return;
                    }

                    self.selected_title_id = Some(title.id.clone());

                    // Only fetch image if it's a different title
                    if self.last_fetched_title_id.as_ref() != Some(&title.id) {
                        // Clear previous image data
                        *self.selected_title_image_bytes.lock().unwrap() = None;
                        self.last_fetched_title_id = Some(title.id.clone());

                        let selected_title_image_bytes =
                            Arc::clone(&self.selected_title_image_bytes);
                        let title_id = title.id.clone();

                        thread::spawn(move || {
                            let mut easy = Easy::new();
                            let mut image_bytes = Vec::new();
                            easy.url(&format!("https://tinfoil.io/ti/{}/200/200", title_id))
                                .unwrap();
                            {
                                let mut transfer = easy.transfer();
                                transfer
                                    .write_function(|data| {
                                        image_bytes.extend_from_slice(data);
                                        Ok(data.len())
                                    })
                                    .unwrap();
                                transfer.perform().unwrap();
                            }
                            *selected_title_image_bytes.lock().unwrap() = Some(image_bytes);
                        });

                        self.active_tab = Tab::CheatManager;
                    }
                });
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.active_tab,
                    Tab::DatabaseExplorer,
                    "Database Explorer",
                );
                ui.selectable_value(&mut self.active_tab, Tab::CheatManager, "Cheat Manager");
            });
            match self.active_tab {
                Tab::DatabaseExplorer => self.table_ui(ui),
                Tab::CheatManager => self.title_ui(ui),
            }
        });
    }
}

fn main() {
    let _ = eframe::run_native(
        concat!(env!("CARGO_PKG_NAME"), " - v", env!("CARGO_PKG_VERSION")),
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_resizable(true)
                .with_inner_size([800.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
}
