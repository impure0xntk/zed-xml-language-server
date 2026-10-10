// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::Path};

use serde_json::Value;
use zed_extension_api::{
    self as zed, Extension, LanguageServerId, LanguageServerInstallationStatus, Worktree,
    register_extension, settings::LspSettings,
};

struct XmlLanguageServerExtension {
    cached_binary: Option<String>,
}

const NAME: &str = "lemminx";
const RED_HAT_REPO: &str = "redhat-developer/vscode-xml";

impl XmlLanguageServerExtension {
    fn new() -> Self {
        Self {
            cached_binary: None,
        }
    }

    fn lsp_settings(&self, worktree: &Worktree) -> Result<LspSettings, String> {
        LspSettings::for_worktree(NAME, worktree)
            .map_err(|e| format!("Failed to read LSP settings for {NAME}: {e}"))
    }

    fn get_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String, String> {
        // 1. Honor an explicitly configured binary path.
        // Do not silently launch a different binary when this path is invalid.
        let settings = self.lsp_settings(worktree)?;

        if let Some(binary) = settings.binary.as_ref()
            && let Some(path) = binary.path.as_deref()
        {
            let path = Path::new(path);

            if !path.is_file() {
                return Err(format!(
                    "Configured LemMinX binary does not exist or is not a file: {}",
                    path.display()
                ));
            }

            return Ok(path.to_string_lossy().into_owned());
        }

        // 2. Prefer a system-managed binary, e.g. one installed by Nix.
        if let Some(path) = worktree.which(NAME) {
            return Ok(path);
        }

        // 3. Reuse the binary resolved by this extension instance.
        if let Some(path) = self.cached_binary.as_deref()
            && Path::new(path).is_file()
        {
            return Ok(path.to_owned());
        }

        // 4. Download the official native binary.
        let path = self.install_binary(language_server_id)?;
        self.cached_binary = Some(path.clone());

        Ok(path)
    }

    fn install_binary(&mut self, language_server_id: &LanguageServerId) -> Result<String, String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            RED_HAT_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let version = release.version.trim_start_matches('v');

        // Keep the release version from becoming a path component.
        if version.is_empty()
            || version.contains('/')
            || version.contains('\\')
            || version == "."
            || version == ".."
        {
            return Err(format!("Invalid LemMinX release version: {version}"));
        }

        let (platform, arch) = zed::current_platform();

        // Use the official release asset names instead of constructing
        // every name from the same OS/architecture pattern.
        let (asset_name, extracted_names): (&str, &[&str]) = match (platform, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => {
                ("lemminx-linux-x86_64.zip", &["lemminx-linux-x86_64"])
            }
            (zed::Os::Linux, zed::Architecture::Aarch64) => {
                ("lemminx-linux-aarch_64.zip", &["lemminx-linux-aarch_64"])
            }
            (zed::Os::Mac, zed::Architecture::X8664) => {
                ("lemminx-osx-x86_64.zip", &["lemminx-osx-x86_64"])
            }
            (zed::Os::Mac, zed::Architecture::Aarch64) => {
                ("lemminx-osx-aarch_64.zip", &["lemminx-osx-aarch_64"])
            }
            (zed::Os::Windows, zed::Architecture::X8664) => {
                ("lemminx-win32.zip", &["lemminx-win32.exe", "lemminx.exe"])
            }
            _ => {
                return Err(format!(
                    "No supported LemMinX binary for {platform:?} {arch:?}"
                ));
            }
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "LemMinX release {} does not contain asset {}",
                    release.version, asset_name
                )
            })?;

        let version_dir = format!("{NAME}-{version}");

        // Reuse an existing installation of this exact version.
        if let Some(path) = Self::find_extracted_binary(Path::new(&version_dir), extracted_names) {
            zed::make_file_executable(&path)?;
            return Ok(path);
        }

        // Remove only our own incomplete version directory.
        // Never delete unrelated entries from the extension directory.
        if Path::new(&version_dir).exists() {
            fs::remove_dir_all(&version_dir).map_err(|e| {
                format!(
                    "Failed to remove incomplete LemMinX directory {}: {e}",
                    version_dir
                )
            })?;
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );

        zed::download_file(
            &asset.download_url,
            &version_dir,
            zed::DownloadedFileType::Zip,
        )?;

        let binary_path = Self::find_extracted_binary(Path::new(&version_dir), extracted_names)
            .ok_or_else(|| {
                format!(
                    "Downloaded {} but could not find the expected executable in {}",
                    asset_name, version_dir
                )
            })?;

        zed::make_file_executable(&binary_path)?;

        Ok(binary_path)
    }

    fn find_extracted_binary(directory: &Path, names: &[&str]) -> Option<String> {
        for name in names {
            let path = directory.join(name);

            if path.is_file() {
                return Some(path.to_string_lossy().into_owned());
            }
        }

        None
    }

    fn server_args(&self, worktree: &Worktree) -> Vec<String> {
        self.lsp_settings(worktree)
            .ok()
            .and_then(|settings| settings.binary)
            .and_then(|binary| binary.arguments)
            .unwrap_or_default()
    }

    fn server_env(&self, worktree: &Worktree) -> Vec<(String, String)> {
        let mut env = Vec::new();

        // Preserve the environment provided by the worktree.
        for (key, value) in worktree.shell_env() {
            if !env.iter().any(|(existing, _)| existing == &key) {
                env.push((key, value));
            }
        }

        // Explicit LSP environment settings override inherited values.
        if let Ok(settings) = self.lsp_settings(worktree)
            && let Some(binary) = settings.binary
            && let Some(overrides) = binary.env
        {
            for (key, value) in overrides {
                if let Some((_, existing_value)) =
                    env.iter_mut().find(|(existing, _)| existing == &key)
                {
                    *existing_value = value;
                } else {
                    env.push((key, value));
                }
            }
        }

        env
    }

    fn initialization_options(&self, worktree: &Worktree) -> Option<Value> {
        self.lsp_settings(worktree)
            .ok()
            .and_then(|settings| settings.initialization_options)
    }

    fn workspace_configuration(&self, worktree: &Worktree) -> Option<Value> {
        self.lsp_settings(worktree)
            .ok()
            .and_then(|settings| settings.settings)
    }
}

impl Extension for XmlLanguageServerExtension {
    fn new() -> Self {
        XmlLanguageServerExtension::new()
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command, String> {
        if language_server_id.as_ref() != NAME {
            return Err(format!(
                "Unknown language server: {}",
                language_server_id.as_ref()
            ));
        }

        let command = self.get_binary(language_server_id, worktree)?;
        let args = self.server_args(worktree);
        let env = self.server_env(worktree);

        Ok(zed::Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>, String> {
        if language_server_id.as_ref() != NAME {
            return Ok(None);
        }

        Ok(self.initialization_options(worktree))
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>, String> {
        if language_server_id.as_ref() != NAME {
            return Ok(None);
        }

        Ok(self.workspace_configuration(worktree))
    }
}

register_extension!(XmlLanguageServerExtension);
