#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use chrono::prelude::*;

pub mod firewall;
pub mod provider;
pub mod settings;
pub mod state;

pub type LastModified = Option<DateTime<FixedOffset>>;
