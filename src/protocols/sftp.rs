use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use ssh2::{CheckResult, HashType, KnownHostFileKind, Session};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use super::RemoteFs;
use crate::domain::file_entry::{EntryKind, FileEntry};

pub struct SftpClient {
    session: Session,
}

impl SftpClient {
    /// Подключение по паролю
    pub fn connect_password(host: &str, port: u16, user: &str, password: &str) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{}:{}", host, port)).context("TCP connect failed")?;
        let mut session = Session::new().context("SSH session init failed")?;
        session.set_tcp_stream(tcp);
        session.handshake().context("SSH handshake failed")?;
        verify_host_key(&session, host, port)?;
        session
            .userauth_password(user, password)
            .context("SSH password auth failed")?;
        Ok(Self { session })
    }

    /// Подключение по ключу
    pub fn connect_key(host: &str, port: u16, user: &str, key_path: &str) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{}:{}", host, port)).context("TCP connect failed")?;
        let mut session = Session::new().context("SSH session init failed")?;
        session.set_tcp_stream(tcp);
        session.handshake().context("SSH handshake failed")?;
        verify_host_key(&session, host, port)?;
        session
            .userauth_pubkey_file(user, None, Path::new(key_path), None)
            .context("SSH key auth failed")?;
        Ok(Self { session })
    }
}

fn known_hosts_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".ssh/known_hosts"))
}

fn fingerprint_hex(session: &Session) -> String {
    session
        .host_key_hash(HashType::Sha256)
        .map(|h| {
            h.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(":")
        })
        .unwrap_or_else(|| "unknown".into())
}

fn verify_host_key(session: &Session, host: &str, port: u16) -> Result<()> {
    let mut known = session.known_hosts().context("known_hosts init failed")?;

    if let Some(ref path) = known_hosts_path() {
        let _ = known.read_file(path, KnownHostFileKind::OpenSSH);
    }

    let (key, key_type) = session
        .host_key()
        .context("no host key received from server")?;

    match known.check_port(host, port, key) {
        CheckResult::Match => {} // known and verified
        CheckResult::Mismatch => {
            bail!(
                "SSH host key mismatch for {}!\n\
                 The server's host key has changed since the last connection.\n\
                 This could mean someone is intercepting the connection (MITM attack).\n\
                 Fingerprint (SHA256): {}",
                host,
                fingerprint_hex(session)
            );
        }
        CheckResult::NotFound => {
            tracing::info!(
                "Unknown host key for {}, adding to known_hosts (SHA256: {})",
                host,
                fingerprint_hex(session)
            );
            known
                .add(host, key, "loflum", key_type.into())
                .context("failed to add host key to known_hosts")?;
            if let Some(ref path) = known_hosts_path()
                && let Err(e) = known.write_file(path, KnownHostFileKind::OpenSSH)
            {
                tracing::warn!("failed to write known_hosts: {}", e);
            }
        }
        CheckResult::Failure => {
            bail!("known_hosts check failed for {}", host);
        }
    }

    Ok(())
}

#[async_trait]
impl RemoteFs for SftpClient {
    async fn list(&self, path: &str) -> Result<Vec<FileEntry>> {
        // TODO: перенести в spawn_blocking когда будет Arc<Session>
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        let entries = sftp.readdir(Path::new(path)).context("readdir failed")?;

        let result = entries
            .into_iter()
            .map(|(pb, stat)| {
                let kind = if stat.is_dir() {
                    EntryKind::Dir
                } else if stat.file_type().is_symlink() {
                    EntryKind::Symlink
                } else {
                    EntryKind::File
                };
                FileEntry {
                    name: pb
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: pb.to_string_lossy().to_string(),
                    kind,
                    size: stat.size,
                    modified: stat.mtime.map(|t| t as i64),
                    permissions: None,
                }
            })
            .collect();

        Ok(result)
    }

    async fn upload(&self, local: &str, remote: &str) -> Result<()> {
        // TODO: chunked upload с прогресс-событиями
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        let mut local_file = std::fs::File::open(local).context("open local file failed")?;
        let mut remote_file = sftp
            .create(Path::new(remote))
            .context("create remote file failed")?;
        std::io::copy(&mut local_file, &mut remote_file)?;
        Ok(())
    }

    async fn download(&self, remote: &str, local: &str) -> Result<()> {
        // TODO: chunked download с прогресс-событиями
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        let mut remote_file = sftp
            .open(Path::new(remote))
            .context("open remote file failed")?;
        let mut local_file = std::fs::File::create(local).context("create local file failed")?;
        std::io::copy(&mut remote_file, &mut local_file)?;
        Ok(())
    }

    async fn mkdir(&self, path: &str) -> Result<()> {
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        sftp.mkdir(Path::new(path), 0o755).context("mkdir failed")?;
        Ok(())
    }

    async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        sftp.rename(Path::new(from), Path::new(to), None)
            .context("rename failed")?;
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        // пробуем как файл, потом как директорию
        if sftp.unlink(Path::new(path)).is_err() {
            sftp.rmdir(Path::new(path)).context("delete failed")?;
        }
        Ok(())
    }

    async fn stat(&self, path: &str) -> Result<FileEntry> {
        let sftp = self.session.sftp().context("SFTP subsystem failed")?;
        let stat = sftp.stat(Path::new(path)).context("stat failed")?;
        Ok(FileEntry {
            name: Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            path: path.to_string(),
            kind: if stat.is_dir() {
                EntryKind::Dir
            } else {
                EntryKind::File
            },
            size: stat.size,
            modified: stat.mtime.map(|t| t as i64),
            permissions: None,
        })
    }
}
