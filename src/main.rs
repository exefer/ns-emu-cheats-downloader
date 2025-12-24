#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod cheat_provider;
mod curl_helper;
mod tinfoil;
mod utils;

use cheat_provider::{CheatMap, CheatSource};
use curl::easy::Easy;
use curl_helper::BodyExt;
use eframe::egui;
use egui::vec2;
use egui_extras::{Column, TableBuilder};
use std::{
    borrow::Cow,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use tinfoil::{TinfoilRoot, TinfoilTitle};

#[derive(PartialEq)]
enum Tab {
    DatabaseExplorer,
    CheatManager,
}

enum InstallMode<'a> {
    Together(&'a Path),
    Separate(&'a Path),
}

struct Settings {
    mods_path: Option<PathBuf>,
}

struct App {
    active_tab: Tab,
    selected_cheat_source: CheatSource,
    selected_title: Option<TinfoilTitle>,
    selected_title_build_id: Option<String>,
    selected_title_cheats: Arc<Mutex<Option<CheatMap>>>,
    selected_title_image_bytes: Arc<Mutex<Option<Vec<u8>>>>,
    settings: Settings,
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
            selected_cheat_source: CheatSource::Blawar,
            selected_title: None,
            selected_title_build_id: None,
            selected_title_image_bytes: Arc::new(Mutex::new(None)),
            selected_title_cheats: Arc::new(Mutex::new(None)),
            settings: Settings { mods_path: None },
            titles,
            titles_filter: String::new(),
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

    fn title_ui(&mut self, ui: &mut egui::Ui) {
        let Some(ref title) = self.selected_title else {
            ui.label("No title selected. Please go to Database Explorer and select a title.");
            return;
        };
        ui.add_space(5.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                if let Some(ref image_bytes) = *self.selected_title_image_bytes.lock().unwrap() {
                    ui.add(
                        egui::Image::new(egui::ImageSource::Bytes {
                            uri: Cow::Owned(format!("bytes://{}.jpeg", title.id)),
                            bytes: egui::load::Bytes::Shared(Arc::from(image_bytes.as_slice())),
                        })
                        .max_size(vec2(235.0, 235.0)),
                    );
                } else {
                    ui.add(egui::Spinner::new().size(235.0));
                }
                ui.add_space(2.5);
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
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("Cheat Source")
                        .selected_text(self.selected_cheat_source.as_str())
                        .show_ui(ui, |ui| {
                            use CheatSource::*;
                            let s = &mut self.selected_cheat_source;
                            ui.selectable_value(s, Blawar, Blawar.as_str());
                            ui.selectable_value(s, Chansey, Chansey.as_str());
                            ui.selectable_value(s, CheatSlips, CheatSlips.as_str());
                            ui.selectable_value(s, GbaTemp, GbaTemp.as_str());
                            ui.selectable_value(s, Hamlet, Hamlet.as_str());
                            ui.selectable_value(s, Ibnux, Ibnux.as_str());
                            ui.selectable_value(s, Tinfoil, Tinfoil.as_str());
                        });
                    if ui.button("Fetch").clicked() {
                        self.selected_title_build_id = None;
                        let selected_title_cheats = Arc::clone(&self.selected_title_cheats);
                        let provider = self.selected_cheat_source.provider();
                        let title_name = title.name.clone();
                        let title_id = title.id.clone();
                        thread::spawn(move || {
                            let cheats = provider.get_cheats_for_title(&title_name, &title_id);
                            *selected_title_cheats.lock().unwrap() =
                                cheats.filter(|c| !c.is_empty());
                        });
                    }
                });
                let Some(ref cheats) = *self.selected_title_cheats.lock().unwrap() else {
                    return;
                };
                ui.add_space(10.0);
                ui.heading("Builds");
                TableBuilder::new(ui)
                    .id_salt("builds")
                    .column(Column::exact(150.0))
                    .column(Column::auto())
                    .striped(true)
                    .sense(egui::Sense::click())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Build ID");
                        });
                        header.col(|ui| {
                            ui.label("Cheats count");
                        });
                    })
                    .body(|body| {
                        let entries: Vec<_> = cheats.iter().collect();
                        body.rows(18.0, entries.len() + 1, |mut row| {
                            if row.index() < entries.len() {
                                let (build_id, cheats) = entries[row.index()];
                                row.col(|ui| {
                                    if self.selected_title_build_id.as_ref() != Some(build_id) {
                                        ui.label(build_id);
                                    } else {
                                        ui.label(egui::RichText::new(build_id).strong());
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(cheats.len().to_string());
                                });
                                if row.response().clicked() {
                                    self.selected_title_build_id = Some(build_id.clone());
                                }
                            } else {
                                row.col(|ui| {
                                    if self.selected_title_build_id.is_some() {
                                        ui.label("All");
                                    } else {
                                        ui.label(egui::RichText::new("All").strong());
                                    }
                                });
                                row.col(|ui| {
                                    let total: usize = entries.iter().map(|(_, c)| c.len()).sum();
                                    ui.label(total.to_string());
                                });
                                if row.response().clicked() {
                                    self.selected_title_build_id = None;
                                }
                            }
                        });
                    });
            });
            if self.selected_title_cheats.lock().unwrap().is_none() {
                return;
            }
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Cheats");
                    ui.add_space(150.0);
                    for (label, checked) in [("Select All", true), ("Deselect All", false)] {
                        if ui.button(label).clicked() {
                            let Some(ref mut cheats) = *self.selected_title_cheats.lock().unwrap()
                            else {
                                return;
                            };
                            if let Some(ref build_id) = self.selected_title_build_id
                                && let Some(cheats) = cheats.get_mut(build_id)
                            {
                                cheats.iter_mut().for_each(|c| c.checked = checked);
                            } else {
                                cheats
                                    .values_mut()
                                    .flatten()
                                    .for_each(|c| c.checked = checked);
                            }
                        }
                    }
                });
                TableBuilder::new(ui)
                    .id_salt("cheats")
                    .column(Column::exact(12.5))
                    .column(Column::exact(600.0))
                    .striped(true)
                    .body(|body| {
                        let Some(ref mut cheats) = *self.selected_title_cheats.lock().unwrap()
                        else {
                            return;
                        };
                        let mut cheats: Vec<_> =
                            if let Some(ref build_id) = self.selected_title_build_id {
                                cheats
                                    .get_mut(build_id)
                                    .map(|c| c.iter_mut().collect())
                                    .unwrap_or_default()
                            } else {
                                cheats.values_mut().flatten().collect()
                            };
                        body.rows(18.0, cheats.len(), |mut row| {
                            let Some(cheat) = cheats.get_mut(row.index()) else {
                                return;
                            };
                            row.col(|ui| {
                                ui.checkbox(&mut cheat.checked, "");
                            });
                            row.col(|ui| {
                                ui.label(&cheat.name);
                            });
                        });
                    });
            });
            let Some(ref cheats) = *self.selected_title_cheats.lock().unwrap() else {
                return;
            };
            ui.horizontal(|ui| {
                let Some(ref mods_path) = self.settings.mods_path else {
                    ui.label("No mod data location selected; select one in the File menu.");
                    return;
                };
                let write_cheats = |mode: InstallMode| {
                    for (build_id, cheats) in cheats {
                        for cheat in cheats.iter().filter(|c| c.checked) {
                            let path = match mode {
                                InstallMode::Together(base) => base.to_path_buf(),
                                InstallMode::Separate(base) => base.join(&cheat.name),
                            }
                            .join("cheats");
                            fs::create_dir_all(&path).ok();
                            let Ok(mut file) =
                                fs::File::create(path.join(format!("{}.txt", build_id)))
                            else {
                                continue;
                            };
                            writeln!(file, "[{}]", cheat.name).ok();
                            for line in &cheat.code {
                                writeln!(file, "{line}").ok();
                            }
                            writeln!(file).ok();
                        }
                    }
                };
                if ui.button("Install Together").clicked()
                    && cheats.values().flatten().any(|c| c.checked)
                {
                    let base = mods_path
                        .join(&title.id)
                        .join(self.selected_cheat_source.as_str());
                    write_cheats(InstallMode::Together(&base));
                }
                if ui.button("Install Separately").clicked() {
                    let base = mods_path.join(&title.id);
                    write_cheats(InstallMode::Separate(&base));
                }
            });
        });
    }

    fn table_ui(&mut self, ui: &mut egui::Ui) {
        let titles = self.titles.lock().unwrap();
        if titles.is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.add(egui::Spinner::new().size(50.0));
                },
            );
            return;
        }
        ui.add_space(2.5);
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.titles_filter);
        });
        ui.add_space(2.5);
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
                    if self.selected_title.as_ref().map(|t| &t.id) != Some(&title.id) {
                        self.selected_title = Some((*title).clone());
                        self.selected_title_build_id = None;
                        *self.selected_title_cheats.lock().unwrap() = None;
                        *self.selected_title_image_bytes.lock().unwrap() = None;
                        let selected_title_image_bytes =
                            Arc::clone(&self.selected_title_image_bytes);
                        let title_id = title.id.clone();
                        thread::spawn(move || {
                            let mut easy = Easy::new();
                            let mut image_bytes = Vec::new();
                            easy.url(&format!("https://tinfoil.io/ti/{}/235/235", title_id))
                                .unwrap();
                            easy.useragent(env!("CARGO_PKG_NAME")).unwrap();
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
                    }
                    self.active_tab = Tab::CheatManager;
                });
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Set Mod Data Location").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_folder()
                    {
                        self.settings.mods_path = Some(path);
                    }
                });
            });
        });
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
