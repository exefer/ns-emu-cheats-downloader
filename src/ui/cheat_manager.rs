use std::borrow::Cow;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use curl::easy::Easy;
use eframe::egui;
use egui::vec2;
use egui_extras::{Column, TableBuilder};

use crate::cheat_provider::{CheatMap, CheatSource};
use crate::tinfoil::TinfoilTitle;

#[derive(Default)]
pub struct CheatManager {
    pub title: Option<TinfoilTitle>,
    pub cheat_source: CheatSource,
    pub build_id: Option<String>,
    pub cheats: Arc<Mutex<Option<CheatMap>>>,
    pub icon: Arc<Mutex<Option<Arc<[u8]>>>>,
}

impl CheatManager {
    pub fn show(&mut self, ui: &mut egui::Ui, folder: Option<&PathBuf>) {
        if self.title.is_none() {
            ui.label("No title selected. Please go to Database Explorer and select a title.");
            return;
        }

        ui.add_space(5.0);
        ui.horizontal_top(|ui| {
            self.show_title_info(ui);
            self.show_cheats_table(ui);
            self.show_install_buttons(ui, folder);
        });
    }

    pub fn set_title(&mut self, title: TinfoilTitle) {
        if self.title.as_ref().map(|t| &t.id) == Some(&title.id) {
            return;
        }

        let title_id = title.id.clone();

        self.title = Some(title);
        self.build_id = None;
        *self.cheats.lock().unwrap() = None;
        *self.icon.lock().unwrap() = None;

        let icon = Arc::clone(&self.icon);

        thread::spawn(move || {
            let mut easy = Easy::new();
            let mut bytes = Vec::new();
            easy.url(&format!("https://tinfoil.io/ti/{}/235/235", title_id))
                .unwrap();
            easy.useragent(env!("CARGO_PKG_NAME")).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        bytes.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .unwrap();
                transfer.perform().unwrap();
            }
            *icon.lock().unwrap() = Some(Arc::from(bytes));
        });
    }

    fn show_title_info(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let title = self.title.as_ref().unwrap();
            if let Some(icon) = self.icon.lock().unwrap().clone() {
                ui.add(
                    egui::Image::new(egui::ImageSource::Bytes {
                        uri: Cow::Owned(format!("bytes://{}.jpeg", title.id)),
                        bytes: egui::load::Bytes::Shared(icon),
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
                    .selected_text(self.cheat_source.as_str())
                    .show_ui(ui, |ui| {
                        use CheatSource::*;
                        let s = &mut self.cheat_source;
                        ui.selectable_value(s, Blawar, Blawar.as_str());
                        ui.selectable_value(s, Chansey, Chansey.as_str());
                        ui.selectable_value(s, CheatSlips, CheatSlips.as_str());
                        ui.selectable_value(s, GbaTemp, GbaTemp.as_str());
                        ui.selectable_value(s, Hamlet, Hamlet.as_str());
                        ui.selectable_value(s, Ibnux, Ibnux.as_str());
                        ui.selectable_value(s, Tinfoil, Tinfoil.as_str());
                    });
                if ui.button("Fetch").clicked() {
                    self.fetch_cheats();
                }
            });

            if self.cheats.lock().unwrap().is_some() {
                self.show_builds_table(ui);
            }
        });
    }

    fn fetch_cheats(&mut self) {
        let title = self.title.as_ref().unwrap();
        self.build_id = None;
        let cheats = Arc::clone(&self.cheats);
        let provider = self.cheat_source.provider();
        let title_name = title.name.clone();
        let title_id = title.id.clone();
        thread::spawn(move || {
            *cheats.lock().unwrap() = provider
                .get_cheats_for_title(&title_name, &title_id)
                .filter(|c| !c.is_empty());
        });
    }

    fn show_builds_table(&mut self, ui: &mut egui::Ui) {
        let Some(cheats) = &*self.cheats.lock().unwrap() else {
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
                            if self.build_id.as_ref() == Some(build_id) {
                                ui.label(egui::RichText::new(build_id).strong());
                            } else {
                                ui.label(build_id);
                            }
                        });
                        row.col(|ui| {
                            ui.label(cheats.len().to_string());
                        });
                        if row.response().clicked() {
                            self.build_id = Some(build_id.clone());
                        }
                    } else {
                        row.col(|ui| {
                            if self.build_id.is_some() {
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
                            self.build_id = None;
                        }
                    }
                });
            });
    }

    fn show_cheats_table(&mut self, ui: &mut egui::Ui) {
        if self.cheats.lock().unwrap().is_none() {
            return;
        }

        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Cheats");
                ui.add_space(150.0);
                for (label, checked) in [("Select All", true), ("Deselect All", false)] {
                    if ui.button(label).clicked() {
                        self.toggle_all_cheats(checked);
                    }
                }
            });

            TableBuilder::new(ui)
                .id_salt("cheats")
                .column(Column::exact(12.5))
                .column(Column::exact(600.0))
                .striped(true)
                .body(|body| {
                    let Some(cheats) = &mut *self.cheats.lock().unwrap() else {
                        return;
                    };

                    let mut cheats: Vec<_> = if let Some(build_id) = &self.build_id {
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
    }

    fn toggle_all_cheats(&mut self, checked: bool) {
        let Some(cheats) = &mut *self.cheats.lock().unwrap() else {
            return;
        };

        if let Some(build_id) = &self.build_id
            && let Some(cheats) = cheats.get_mut(build_id)
        {
            for c in cheats {
                c.checked = checked;
            }
        } else {
            for c in cheats.values_mut().flatten() {
                c.checked = checked;
            }
        }
    }

    fn show_install_buttons(&self, ui: &mut egui::Ui, folder: Option<&PathBuf>) {
        let Some(cheats) = &*self.cheats.lock().unwrap() else {
            return;
        };
        let title = self.title.as_ref().unwrap();

        ui.horizontal(|ui| {
            let Some(folder) = folder else {
                ui.label("No mod data location selected; select one in the File menu.");
                return;
            };

            if ui.button("Install Together").clicked()
                && cheats.values().flatten().any(|c| c.checked)
            {
                let base = folder.join(&title.id).join(self.cheat_source.as_str());
                Self::write_cheats_together(&base, cheats);
            }

            if ui.button("Install Separately").clicked() {
                let base = folder.join(&title.id);
                Self::write_cheats_separately(&base, cheats);
            }
        });
    }

    fn write_cheats_together(base: &Path, cheats: &CheatMap) {
        let path = base.join("cheats");
        create_dir_all(&path).ok();

        for (build_id, cheats) in cheats {
            let Ok(mut file) = File::create(path.join(format!("{}.txt", build_id))) else {
                continue;
            };
            for cheat in cheats.iter().filter(|c| c.checked) {
                writeln!(file, "[{}]", cheat.name).ok();
                for line in &cheat.code {
                    writeln!(file, "{}", line).ok();
                }
                writeln!(file).ok();
            }
        }
    }

    fn write_cheats_separately(base: &Path, cheats: &CheatMap) {
        for (build_id, cheats) in cheats {
            for cheat in cheats.iter().filter(|c| c.checked) {
                let path = base.join(&cheat.name).join("cheats");
                create_dir_all(&path).ok();
                let Ok(mut file) = File::create(path.join(format!("{}.txt", build_id))) else {
                    continue;
                };
                writeln!(file, "[{}]", cheat.name).ok();
                for line in &cheat.code {
                    writeln!(file, "{}", line).ok();
                }
            }
        }
    }
}
