use std::{collections::BTreeMap, fs, io::ErrorKind, path::PathBuf};

use anyhow::{Context, Result};
use chrono::prelude::*;
use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub last_modified: BTreeMap<String, DateTime<FixedOffset>>,
}

impl State {
    pub fn load() -> Result<Self> {
        let path = get_path()?;

        let content = match fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };

        toml::from_slice(&content).map_err(Into::into)
    }

    pub fn save(&self) -> Result<()> {
        let path = get_path()?;

        fs::create_dir_all(path.parent().context("no parent path")?)?;

        let content = toml::to_string_pretty(self)?;

        fs::write(path, content).map_err(Into::into)
    }
}

fn get_path() -> Result<PathBuf> {
    Ok(
        ProjectDirs::from("rocks", "dnaka91", env!("CARGO_PKG_NAME"))
            .context("Project dirs not found")?
            .data_dir()
            .join("state.toml"),
    )
}
