use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        let add_btn = egui::Button::new("+ New Connection")
            .fill(egui::Color32::from_rgb(80, 60, 180));
        if ui.add(add_btn).clicked() {
            state.show_connect_dialog = true;
        }
    });
}
