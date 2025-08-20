use crate::error::{JSResult, ScriptError};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct ScriptConfig {
    pub scripts_dir: PathBuf,
    pub auto_execute: bool,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        let scripts_dir = std::env::current_exe()
            .ok()
            .and_then(|exe_path| exe_path.parent().map(|p| p.join("scripts")))
            .unwrap_or_else(|| PathBuf::from("scripts"));

        Self {
            scripts_dir,
            auto_execute: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptManager {
    pub scripts_dir: PathBuf,
    pub data_scripts: Vec<PathBuf>,
    pub html_scripts: Vec<PathBuf>,
}

impl ScriptManager {
    pub fn new(config: ScriptConfig) -> JSResult<Self> {
        let mut manager = Self {
            scripts_dir: config.scripts_dir,
            data_scripts: Vec::new(),
            html_scripts: Vec::new(),
        };

        if config.auto_execute {
            manager.discover_scripts()?;
        }

        Ok(manager)
    }

    pub fn discover_scripts(&mut self) -> JSResult<()> {
        self.data_scripts.clear();
        self.html_scripts.clear();

        if !self.scripts_dir.exists() {
            debug!("Scripts directory does not exist: {:?}", self.scripts_dir);
            return Ok(());
        }

        let data_dir = self.scripts_dir.join("data");
        if data_dir.exists() && data_dir.is_dir() {
            for entry in std::fs::read_dir(&data_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("js") {
                    debug!("Found data script: {:?}", path);
                    self.data_scripts.push(path);
                }
            }
        }

        let html_dir = self.scripts_dir.join("html");
        if html_dir.exists() && html_dir.is_dir() {
            for entry in std::fs::read_dir(&html_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("js") {
                    debug!("Found HTML script: {:?}", path);
                    self.html_scripts.push(path);
                }
            }
        }

        self.data_scripts.sort();
        self.html_scripts.sort();

        debug!(
            "Discovered {} data scripts and {} HTML scripts",
            self.data_scripts.len(),
            self.html_scripts.len()
        );

        Ok(())
    }

    pub fn has_data_scripts(&self) -> bool {
        !self.data_scripts.is_empty()
    }

    pub fn has_html_scripts(&self) -> bool {
        !self.html_scripts.is_empty()
    }

    pub fn get_data_scripts(&self) -> &[PathBuf] {
        &self.data_scripts
    }

    pub fn get_html_scripts(&self) -> &[PathBuf] {
        &self.html_scripts
    }

    /// Add a single data processing script
    pub fn add_data_script(&mut self, script_path: PathBuf) -> JSResult<()> {
        if !script_path.exists() {
            return Err(ScriptError::FileNotFound(script_path));
        }

        if script_path.extension().and_then(|s| s.to_str()) != Some("js") {
            return Err(ScriptError::InvalidOutput(
                "Script file must have .js extension".to_string(),
            ));
        }

        self.data_scripts.push(script_path);
        self.data_scripts.sort();
        Ok(())
    }

    /// Add a single HTML processing script
    pub fn add_html_script(&mut self, script_path: PathBuf) -> JSResult<()> {
        if !script_path.exists() {
            return Err(ScriptError::FileNotFound(script_path));
        }

        if script_path.extension().and_then(|s| s.to_str()) != Some("js") {
            return Err(ScriptError::InvalidOutput(
                "Script file must have .js extension".to_string(),
            ));
        }

        self.html_scripts.push(script_path);
        self.html_scripts.sort();
        Ok(())
    }

    pub fn refresh(&mut self) -> JSResult<()> {
        self.discover_scripts()
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new(ScriptConfig::default()).expect("Failed to create default ScriptManager")
    }
}
