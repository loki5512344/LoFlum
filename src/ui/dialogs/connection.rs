use crate::domain::connection::{ConnectionParams, Protocol};
use crate::domain::file_entry::FileEntry;
use crate::fs::remote::RemoteRegistry;
use crate::protocols::{RemoteFs, ftp::FtpClient, sftp::SftpClient};
use crate::storage::keychain;
use crate::ui::state::{AppState, PendingConnect};
use std::sync::Arc;

pub fn render(
    ctx: &egui::Context,
    state: &mut AppState,
    registry: &Arc<RemoteRegistry>,
    rt_handle: &tokio::runtime::Handle,
) {
    let mut open = true;
    egui::Window::new("New Connection")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Protocol:");
                let protocols = ["SFTP", "FTP", "FTPS"];
                egui::ComboBox::from_id_source("protocol")
                    .selected_text(protocols[state.connect_protocol])
                    .show_ui(ui, |ui| {
                        for (i, p) in protocols.iter().enumerate() {
                            ui.selectable_value(&mut state.connect_protocol, i, *p);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Label:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.connect_label)
                        .id("label".into())
                        .desired_width(200.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.connect_host)
                        .id("host".into())
                        .desired_width(200.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.connect_port)
                        .id("port".into())
                        .desired_width(60.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.connect_user)
                        .id("user".into())
                        .desired_width(200.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.connect_pass)
                        .password(true)
                        .id("pass".into())
                        .desired_width(200.0),
                );
            });

            if state.connect_protocol == 0 {
                ui.horizontal(|ui| {
                    ui.label("Key path:");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.connect_key_path)
                            .id("key".into())
                            .desired_width(200.0),
                    );
                });
            }

            if !state.connect_error.is_empty() {
                ui.colored_label(egui::Color32::RED, &state.connect_error);
            }

            ui.add_enabled_ui(!state.connect_loading, |ui| {
                if ui.button("Connect").clicked() {
                    state.connect_error.clear();

                    let protocol = match state.connect_protocol {
                        0 => Protocol::Sftp,
                        1 => Protocol::Ftp,
                        _ => Protocol::Ftps,
                    };

                    let port: u16 = state.connect_port.parse().unwrap_or(match protocol {
                        Protocol::Sftp => 22,
                        Protocol::Ftp => 21,
                        Protocol::Ftps => 990,
                    });

                    let params = ConnectionParams {
                        id: uuid::Uuid::new_v4().to_string(),
                        label: if state.connect_label.is_empty() {
                            state.connect_host.clone()
                        } else {
                            state.connect_label.clone()
                        },
                        protocol,
                        host: state.connect_host.clone(),
                        port,
                        username: state.connect_user.clone(),
                        password: if state.connect_pass.is_empty() {
                            None
                        } else {
                            Some(state.connect_pass.clone())
                        },
                        key_path: if state.connect_key_path.is_empty() {
                            None
                        } else {
                            Some(state.connect_key_path.clone())
                        },
                    };

                    let registry = registry.clone();
                    let params_clone = params.clone();
                    let result = Arc::new(std::sync::Mutex::new(None));
                    let result_clone = result.clone();

                    state.connect_loading = true;
                    rt_handle.spawn(async move {
                        let r = do_connect(&registry, &params_clone).await;
                        *result_clone.lock().unwrap() = Some(r);
                    });

                    state.pending_connect = Some(PendingConnect { result });
                }
            });

            if state.connect_loading {
                ui.label("Connecting...");
            }
        });

    if !open {
        state.show_connect_dialog = false;
    }
}

async fn do_connect(
    registry: &RemoteRegistry,
    params: &ConnectionParams,
) -> Result<(ConnectionParams, Vec<FileEntry>), String> {
    let password = if let Some(p) = &params.password {
        p.clone()
    } else {
        keychain::get_password(&params.id).map_err(|e| e.to_string())?
    };

    let fs: Arc<dyn RemoteFs> = match params.protocol {
        Protocol::Sftp => {
            if let Some(key_path) = &params.key_path {
                Arc::new(
                    SftpClient::connect_key(&params.host, params.port, &params.username, key_path)
                        .map_err(|e| e.to_string())?,
                )
            } else {
                Arc::new(
                    SftpClient::connect_password(
                        &params.host,
                        params.port,
                        &params.username,
                        &password,
                    )
                    .map_err(|e| e.to_string())?,
                )
            }
        }
        Protocol::Ftp => Arc::new(
            FtpClient::connect(&params.host, params.port, &params.username, &password)
                .await
                .map_err(|e| e.to_string())?,
        ),
        Protocol::Ftps => return Err("FTPS not yet implemented".into()),
    };

    registry.insert(params.id.clone(), fs.clone());

    let entries = fs.list("/").await.map_err(|e| e.to_string())?;
    Ok((params.clone(), entries))
}
