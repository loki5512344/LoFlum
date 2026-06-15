use crate::domain::file_entry::FileEntry;

#[derive(Clone, Debug)]
pub enum DragPayload {
    LocalFile(String, String),
    RemoteFile(String, String, String),
}

impl DragPayload {
    pub fn file_name(&self) -> &str {
        match self {
            DragPayload::LocalFile(_, name) => name,
            DragPayload::RemoteFile(_, name, _) => name,
        }
    }
}

pub fn make_local_payload(entry: &FileEntry) -> Option<DragPayload> {
    if entry.name == ".." {
        return None;
    }
    Some(DragPayload::LocalFile(
        entry.path.clone(),
        entry.name.clone(),
    ))
}

pub fn make_remote_payload(entry: &FileEntry, connection_id: &str) -> Option<DragPayload> {
    if entry.name == ".." {
        return None;
    }
    Some(DragPayload::RemoteFile(
        entry.path.clone(),
        entry.name.clone(),
        connection_id.to_string(),
    ))
}

impl DragPayload {
    pub fn is_from_local(&self) -> bool {
        matches!(self, DragPayload::LocalFile(_, _))
    }

    pub fn is_from_remote(&self) -> bool {
        matches!(self, DragPayload::RemoteFile(_, _, _))
    }
}
