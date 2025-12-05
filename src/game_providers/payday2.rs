use std::{fs, io::{Error, ErrorKind}, path::{Path, PathBuf}};

use async_trait::async_trait;
use lib_vmm::{archive::{ArchiveInfo, determine_root_dir, ensure_dir, extract_zip, inspect_zip, replace_symlink_dir}, registry::ProviderSource, traits::{game_provider::{GameIcon, GameInstallError, GameMetadata, GameProvider}, provider::Provider}};

use crate::helper::staging::StagingGuard;

/// Testing for upstream lib-vmm
pub trait InstallResultExt<T> {
    fn ctx(self, op: &'static str) -> Result<T, GameInstallError>;
}

impl<T, E: std::error::Error> InstallResultExt<T> for Result<T, E> {
    fn ctx(self, op: &'static str) -> Result<T, GameInstallError> {
        self.map_err(|e| {
            GameInstallError::IO(Error::other(
                format!("{op} failed: {e}")
            ))
        })
    }
}

pub struct Payday2Provider;

#[derive(Debug)]
enum ModKind {
    Lua,
    Override
}

impl Payday2Provider {
    pub fn new() -> Self { Self }

    fn resolve_install_path(&self) -> Option<PathBuf> {
        let steam_dir = steamlocate::SteamDir::locate().ok()?;
        let app_id = 218_620;
        let (game, lib) = steam_dir.find_app(app_id).ok()??;
        Some(lib.resolve_app_dir(&game))
    }

    fn mods_folder(&self, base: &Path) -> PathBuf { base.join("mods") }
    fn overrides_folder(&self, base: &Path) -> PathBuf { base.join("assets").join("mod_overrides") }

    fn classify(info: &ArchiveInfo) -> ModKind {
        if info.count_ext("lua") > 1 {
            ModKind::Lua
        } else {
            ModKind::Override
        }
    }
}

impl Provider for Payday2Provider {
    fn id(&self) -> &'static str { "core:payday_2" }

    fn capabilities(&self) -> &[lib_vmm::capabilities::base::CapabilityRef] { &[] }
}

#[async_trait]
impl GameProvider for Payday2Provider {
    fn mod_provider_id(&self) -> &str { "core:modworkshop" }

    fn metadata(&self) -> GameMetadata {
        GameMetadata {
            id: self.id().into(),
            display_name: "PAYDAY 2".into(),
            short_name: "PD2".into(),
            icon: GameIcon::Path("https://cdn2.steamgriddb.com/icon/fa6d3cc166fbfbf005c9e77d96cba283/32/256x256.png".into()),
            provider_source: ProviderSource::Core
        }
    }

    /// Game ID for modworkshop query
    fn get_external_id(&self) -> &str { "1" }
    fn install_mod(&self, target: &Path) -> Result<(), GameInstallError> {
        // Game
        let game_install_path = self.resolve_install_path().ok_or(GameInstallError::MissingGameFiles)?;

        // Mod stuff
        let mod_folder = self.mods_folder(&game_install_path);
        let mod_asset_folder = self.overrides_folder(&game_install_path);

        ensure_dir(&mod_folder).ctx("ensure mods dir")?;
        ensure_dir(&mod_asset_folder).ctx("ensure overrides dir")?;

        // Archive stuff
        let info = inspect_zip(target).ctx("inspect zip")?;
        let inspected_root = info.single_top_level_dir();

        let raw_name_os = inspected_root
            .as_ref()
            .and_then(|p| p.file_name().map(|s| s.to_owned()))
            .unwrap_or_else(|| target.file_stem().unwrap_or_default().to_os_string());

        let extracted_root = dirs::data_local_dir()
            .ok_or_else(|| GameInstallError::IO(Error::new(ErrorKind::NotFound, "Couldn't resolve local data dir")))?
            .join("me.ghoul.void_mod_manager")
            .join("mods")
            .join("extracted")
            .join(self.id())
            .join(&raw_name_os);

        let staging = extracted_root.with_extension("staging");
        let guard = StagingGuard::new(staging.clone());

        // Clean up old staging
        if staging.exists() {
            fs::remove_dir_all(&staging).ctx("remove old staging")?;
        }

        let extracted_info = extract_zip(target, &staging).ctx("extract zip")?;

        let mod_kind = Self::classify(&extracted_info);

        // Remove previous root and rename staging directory to it
        if extracted_root.exists() {
            fs::remove_dir_all(&extracted_root).ctx("remove previous extracted root")?;
        }

        // Rename
        fs::rename(guard.path(), &extracted_root).ctx("rename staging to extracted")?;
        guard.commit();

        // Determine root dir and resolve destination
        let resolved_root = determine_root_dir(&extracted_info, &extracted_root);
        let root_dir = if resolved_root.is_dir() { resolved_root } else { extracted_root.clone() };

        let dest_path = match mod_kind {
            ModKind::Lua => mod_folder.join(&raw_name_os),
            ModKind::Override => mod_asset_folder.join(&raw_name_os),
        };

        replace_symlink_dir(&root_dir, &dest_path).ctx("link mod directory")?;

        Ok(())

    }
}
