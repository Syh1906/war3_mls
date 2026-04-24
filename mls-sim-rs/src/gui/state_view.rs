use eframe::egui;

use super::{section_heading, GuiApp};

impl GuiApp {
    pub(crate) fn state_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                let room_ids: Vec<String> = {
                    let manager = self.manager.read().unwrap();
                    let mut ids: Vec<String> = manager.rooms.keys().cloned().collect();
                    ids.sort();
                    ids
                };

                ui.horizontal(|ui: &mut egui::Ui| {
                    section_heading(ui, "房间状态");

                    ui.add_space(12.0);
                    ui.label("房间:");
                    let selected_text =
                        self.state_room_id.as_deref().unwrap_or("-- 选择房间 --");
                    egui::ComboBox::from_id_salt("state_room_select")
                        .selected_text(selected_text)
                        .width(140.0)
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            for id in &room_ids {
                                let is_selected = self.state_room_id.as_ref() == Some(id);
                                if ui.selectable_label(is_selected, id).clicked() {
                                    self.state_room_id = Some(id.clone());
                                    self.refresh_state_json();
                                }
                            }
                        });

                    if ui.button("刷新").clicked() {
                        self.refresh_state_json();
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                if self.state_room_id.is_none() {
                    ui.vertical_centered(|ui: &mut egui::Ui| {
                        ui.add_space(ui.available_height() * 0.3);
                        ui.label(
                            egui::RichText::new("请选择一个房间查看状态")
                                .size(15.0)
                                .color(egui::Color32::from_rgb(140, 148, 165)),
                        );
                    });
                    return;
                }

                egui::ScrollArea::both().show(ui, |ui: &mut egui::Ui| {
                    let mut text = self.state_json_text.as_str();
                    ui.add(
                        egui::TextEdit::multiline(&mut text)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(35)
                            .code_editor(),
                    );
                });
            });
    }

    fn refresh_state_json(&mut self) {
        if let Some(room_id) = &self.state_room_id {
            let manager = self.manager.read().unwrap();
            if let Some(room) = manager.rooms.get(room_id) {
                let shared = room.shared.read().unwrap();
                let json = shared.to_json();
                self.state_json_text =
                    serde_json::to_string_pretty(&json).unwrap_or_else(|e| format!("Error: {}", e));
            } else {
                self.state_json_text = "房间��存在".to_string();
                self.state_room_id = None;
            }
        }
    }
}
