use crate::domain::transfer::TransferTask;
use crate::ui::panels::file_pane::format_size;
use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, _state: &mut AppState, tasks: &[TransferTask]) {
    ui.horizontal(|ui| {
        ui.strong("Transfer Queue");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{} tasks", tasks.len()));
        });
    });

    ui.separator();

    if tasks.is_empty() {
        ui.label("No pending transfers");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .max_height(200.0)
        .show(ui, |ui| {
            for task in tasks {
                let bg = match &task.state {
                    crate::domain::transfer::TaskState::Completed => {
                        egui::Color32::from_rgb(220, 255, 220)
                    }
                    crate::domain::transfer::TaskState::Failed(_) => {
                        egui::Color32::from_rgb(255, 220, 220)
                    }
                    crate::domain::transfer::TaskState::Running => {
                        egui::Color32::from_rgb(220, 220, 255)
                    }
                    _ => egui::Color32::TRANSPARENT,
                };

                egui::Frame::none()
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let arrow = match &task.kind {
                                crate::domain::transfer::TransferKind::Upload => "\u{2191}",
                                crate::domain::transfer::TransferKind::Download => "\u{2193}",
                            };
                            ui.label(arrow);
                            ui.label(&task.file_name);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(task.state.to_string());
                                },
                            );
                        });

                        ui.horizontal(|ui| {
                            let pct = task.progress_pct();
                            let pb = egui::ProgressBar::new(pct as f32 / 100.0)
                                .text(format!("{:.1}%", pct))
                                .fill(egui::Color32::from_rgb(100, 80, 220))
                                .desired_width(ui.available_width() * 0.6);
                            ui.add(pb);

                            ui.label(format!(
                                "{}/{}",
                                format_size(Some(task.transferred_bytes)),
                                format_size(Some(task.total_bytes))
                            ));
                        });

                        ui.horizontal(|ui| {
                            if let Some(speed) = task.speed {
                                ui.label(format!("{}/s", format_size(Some(speed))));
                            }
                            if let Some(eta) = task.eta_secs {
                                ui.label(format!("ETA: {}s", eta));
                            }
                        });
                    });
            }
        });
}
