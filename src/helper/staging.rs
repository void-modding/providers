/// This is a testing ground for a potential StagingGuard, before merged into lib-vmm
/// This lets us get real usage out of it, before comitting to it.
use std::path::{Path, PathBuf};

pub struct StagingGuard {
    staging: PathBuf,
    comitted: bool,
}

impl StagingGuard {
    pub fn new(staging: PathBuf) -> Self { Self { staging, comitted: false } }
    pub fn path(&self) -> &Path { &self.staging }
    pub fn commit(mut self) { self.comitted = true; }
}

impl Drop for StagingGuard {
    fn drop(&mut self) {
        if !self.comitted && self.staging.exists() {
            let _ = std::fs::remove_dir_all(&self.staging);
        }
    }
}
