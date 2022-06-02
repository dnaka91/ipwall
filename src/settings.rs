use std::{
    collections::BTreeMap,
    fmt::{self, Display},
    fs,
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub target: IptablesTarget,
    #[serde(default)]
    pub firehol: Firehol,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
}

#[derive(Copy, Clone, Deserialize)]
pub enum IptablesTarget {
    Drop,
    Reject,
    Tarpit,
}

impl IptablesTarget {
    #[must_use]
    pub const fn to_args(self) -> &'static [&'static str] {
        match self {
            Self::Drop => &["DROP"],
            Self::Reject => &["REJECT"],
            Self::Tarpit => &["TARPIT", "--tarpit"],
        }
    }
}

impl Default for IptablesTarget {
    fn default() -> Self {
        Self::Drop
    }
}

impl Display for IptablesTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Drop => "DROP",
            Self::Reject => "REJECT",
            Self::Tarpit => "TARPIT --tarpit",
        })
    }
}

#[derive(Deserialize)]
pub struct Firehol {
    pub level1: bool,
    pub level2: bool,
    pub level3: bool,
}

impl Default for Firehol {
    fn default() -> Self {
        Self {
            level1: false,
            level2: true,
            level3: true,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = get_path()?;

        let content = match fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };

        toml::from_slice(&content).map_err(Into::into)
    }
}

fn get_path() -> Result<PathBuf> {
    Ok(
        ProjectDirs::from("rocks", "dnaka91", env!("CARGO_PKG_NAME"))
            .context("Project dirs not found")?
            .data_dir()
            .join("config.toml"),
    )
}
