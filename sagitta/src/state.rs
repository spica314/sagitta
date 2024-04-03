use std::path::PathBuf;

use crate::{server_files_manager::ServerFilesManager, tools::Clock};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub server_files_manager: ServerFilesManager,
    pub clock: Clock,
}

impl ApiState {
    pub fn new(base_path: PathBuf, clock: Clock) -> Self {
        Self {
            server_files_manager: ServerFilesManager::new(base_path),
            clock,
        }
    }
}
