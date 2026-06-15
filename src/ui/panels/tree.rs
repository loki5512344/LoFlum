use crate::domain::file_entry::{EntryKind, FileEntry};
use crate::fs::local;
use crate::ui::state::AppState;

pub fn refresh_tree(state: &mut AppState, path: &str) {
    if state.tree_children.contains_key(path) {
        return;
    }
    state.tree_loading.insert(path.to_string(), true);
    let entries = local::list(path).unwrap_or_default();
    let dirs: Vec<FileEntry> = entries
        .into_iter()
        .filter(|e| e.kind == EntryKind::Dir)
        .collect();
    state.tree_children.insert(path.to_string(), dirs);
    state.tree_loading.remove(path);
}

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.strong("Local Files");
            ui.separator();

            let home = dirs::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let roots = [("/", "Root"), (&home, "Home"), ("/tmp", "tmp")];

            for (root_path, label) in &roots {
                render_tree_node(ui, state, root_path, label, 0);
            }

            ui.separator();
            ui.strong("Bookmarks");
            for bm in &state.bookmarks.clone() {
                let is_current = state.local_path == bm.path
                    || state.local_path.starts_with(&bm.path);
                let mut btn = egui::Button::new(&bm.name).min_size(egui::vec2(140.0, 20.0));
                if is_current {
                    btn = btn.fill(egui::Color32::from_rgb(50, 55, 60));
                }
                if ui.add(btn).clicked() {
                    state.local_path = bm.path.clone();
                    crate::ui::panels::local_pane::refresh_local(state);
                }
            }
        });
}

fn render_tree_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    path: &str,
    label: &str,
    depth: usize,
) {
    let is_current = state.local_path == path
        || (path != "/"
            && state.local_path.starts_with(path)
            && state.local_path.as_str()[path.len()..].starts_with('/'));

    let expanded = state.tree_expanded.get(path).copied().unwrap_or(false);
    let has_children = state.tree_children.contains_key(path);

    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 16.0);

        let arrow = if has_children {
            if expanded {
                "\u{25BC}"
            } else {
                "\u{25B6}"
            }
        } else {
            " "
        };

        let arrow_resp = ui.selectable_label(false, arrow);
        if arrow_resp.clicked() && has_children {
            if expanded {
                state.tree_expanded.insert(path.to_string(), false);
            } else {
                state.tree_expanded.insert(path.to_string(), true);
                if !state.tree_children.contains_key(path) {
                    refresh_tree(state, path);
                }
            }
        }

        let icon = if expanded {
            "\u{1F4C2}"
        } else {
            "\u{1F4C1}"
        };
        let mut btn = egui::Button::new(format!("{} {}", icon, label))
            .min_size(egui::vec2(120.0, 20.0));
        if is_current {
            btn = btn.fill(egui::Color32::from_rgb(50, 55, 60));
        }
        if ui.add(btn).clicked() {
            state.local_path = path.to_string();
            crate::ui::panels::local_pane::refresh_local(state);
        }
    });

    if expanded {
        let children = state.tree_children.get(path).cloned().unwrap_or_default();
        let mut dirs: Vec<_> = children.iter().collect();
        dirs.sort_by(|a, b| a.name.cmp(&b.name));

        for child in &dirs {
            let child_path = child.path.clone();
            let child_label = child.name.clone();

            if !state.tree_children.contains_key(&child_path) {
                refresh_tree(state, &child_path);
            }

            render_tree_node(ui, state, &child_path, &child_label, depth + 1);
        }
    }
}
