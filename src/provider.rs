use std::collections::HashSet;

use anyhow::Result;
use chrono::prelude::*;
use ipnetwork::IpNetwork;

use crate::LastModified;

pub struct Provider<'a> {
    source: &'a Source,
}

impl<'a> Provider<'a> {
    #[must_use]
    pub const fn new(source: &'a Source) -> Self {
        Self { source }
    }

    pub fn get(&self, lm: LastModified) -> Result<(HashSet<IpNetwork>, LastModified)> {
        let resp = ureq::get(self.source.url()).call()?;

        let lm = if let Some(value) = resp.header("Last-Modified") {
            let new_lm = DateTime::parse_from_rfc2822(value)?;

            if let Some(lm) = lm {
                if new_lm < lm {
                    return Ok((HashSet::new(), Some(lm)));
                }
            }

            Some(new_lm)
        } else {
            lm
        };

        let ips = resp
            .into_string()?
            .lines()
            .filter_map(|l| {
                let l = extract_comment(l).trim();
                if l.is_empty() {
                    None
                } else {
                    Some(l.parse().map_err(Into::into))
                }
            })
            .collect::<Result<HashSet<_>>>()?;

        Ok((ips, lm))
    }
}

pub enum Source {
    FireHolLevel1,
    FireHolLevel2,
    FireHolLevel3,
    Custom { name: String, url: String },
}

impl Source {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::FireHolLevel1 => "firehol-level1",
            Self::FireHolLevel2 => "firehol-level2",
            Self::FireHolLevel3 => "firehol-level3",
            Self::Custom { name, .. } => name,
        }
    }

    fn url(&self) -> &str {
        match self {
            Self::FireHolLevel1 => "https://iplists.firehol.org/files/firehol_level1.netset",
            Self::FireHolLevel2 => "https://iplists.firehol.org/files/firehol_level2.netset",
            Self::FireHolLevel3 => "https://iplists.firehol.org/files/firehol_level3.netset",
            Self::Custom { url, .. } => url,
        }
    }
}

fn extract_comment(line: &str) -> &str {
    line.find(|c| c == ';' || c == '#')
        .map_or(line, |index| &line[..index])
}

#[cfg(test)]
mod tests {
    #[test]
    fn comment() {
        assert_eq!(
            "2.56.255.0/24 ",
            super::extract_comment("2.56.255.0/24 ; SBL444288")
        );
    }
}
