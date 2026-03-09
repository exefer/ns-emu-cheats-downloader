use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::tinfoil::TinfoilTitle;

pub struct DatabaseExplorer {
    pub filter: String,
    pub titles: Arc<Mutex<Vec<TinfoilTitle>>>,
}

impl DatabaseExplorer {
    pub fn new(titles: Arc<Mutex<Vec<TinfoilTitle>>>) -> Self {
        Self {
            filter: String::new(),
            titles,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<TinfoilTitle> {
        let titles = self.titles.lock().unwrap();

        if titles.is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.add(egui::Spinner::new().size(50.0));
                },
            );
            return None;
        }

        ui.add_space(2.5);
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.filter);
        });
        ui.add_space(2.5);

        let mut selected_title = None;

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
                let filter = self.filter.as_str();
                let titles: Vec<_> = if filter.is_empty() {
                    titles.iter().collect()
                } else {
                    titles
                        .iter()
                        .filter(|title| title.name.contains(filter) || title.id.contains(filter))
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

                    if row.response().clicked() {
                        selected_title = Some((*title).clone());
                    }
                });
            });

        selected_title
    }
}
