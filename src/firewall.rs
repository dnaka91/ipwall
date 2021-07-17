use std::{collections::HashSet, fmt::Write as FmtWrite, fs, iter, path::PathBuf};

use anyhow::{ensure, Result};
use duct::cmd;
use ipnetwork::IpNetwork;
use rand::{distributions::Alphanumeric, Rng};

use crate::settings::IptablesTarget;

pub trait Firewall {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn block(&self, ips: &HashSet<IpNetwork>) -> Result<()>;
}

pub struct IpSet {
    name: String,
    ipset_path: PathBuf,
    iptables_path: PathBuf,
    target: IptablesTarget,
}

fn find_binary(name: &str, default: &str) -> Result<PathBuf> {
    use std::os::unix::fs::MetadataExt;

    if let Ok(path) = which::which(name) {
        return Ok(path);
    }

    let meta = fs::metadata(default)
        .map(|meta| meta.is_file() && meta.mode() & 0o111 != 0)
        .unwrap_or_default();
    ensure!(meta, "cannot find binary path");

    Ok(PathBuf::from(default))
}

impl IpSet {
    pub fn new(name: &str, target: IptablesTarget) -> Result<Self> {
        Ok(Self {
            name: format!("{}-{}", env!("CARGO_PKG_NAME"), name),
            ipset_path: find_binary("ipset", "/usr/sbin/ipset")?,
            iptables_path: find_binary("iptables", "/usr/sbin/iptables")?,
            target,
        })
    }

    fn random_name() -> String {
        let mut rng = rand::thread_rng();

        iter::repeat(())
            .map(|_| rng.sample(Alphanumeric))
            .map(char::from)
            .take(20)
            .collect()
    }
}

impl Firewall for IpSet {
    fn install(&self) -> Result<()> {
        let output = cmd!(&self.ipset_path, "list", "-n").read()?;

        if !output.lines().any(|l| l == self.name) {
            cmd!(&self.ipset_path, "create", &self.name, "hash:net").run()?;
        }

        let output = cmd!(&self.iptables_path, "-S").read()?;

        for chain in &["INPUT", "FORWARD"] {
            if !output.contains(&format!(
                "-A {} -p tcp -m multiport --dports 22,80,443 -m set --match-set {} src -j {}",
                chain, &self.name, self.target,
            )) {
                let mut args = vec![
                    "-I",
                    chain,
                    "-p",
                    "tcp",
                    "-m",
                    "multiport",
                    "--dports",
                    "22,80,443",
                    "-m",
                    "set",
                    "--match-set",
                    &self.name,
                    "src",
                    "-j",
                ];
                args.extend(self.target.to_args());

                duct::cmd(&self.iptables_path, args).run()?;
            }
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        for chain in &["INPUT", "FORWARD"] {
            loop {
                let mut args = vec![
                    "-D",
                    chain,
                    "-p",
                    "tcp",
                    "-m",
                    "multiport",
                    "--dports",
                    "22,80,443",
                    "-m",
                    "set",
                    "--match-set",
                    &self.name,
                    "src",
                    "-j",
                ];
                args.extend(self.target.to_args());

                let status = duct::cmd(&self.iptables_path, args)
                    .stderr_null()
                    .unchecked()
                    .run()?
                    .status;

                if !status.success() {
                    break;
                }
            }
        }

        cmd!(&self.ipset_path, "destroy", &self.name)
            .unchecked()
            .run()?;

        Ok(())
    }

    fn block(&self, ips: &HashSet<IpNetwork>) -> Result<()> {
        let temp_set = Self::random_name();
        let mut buf = String::new();

        writeln!(
            &mut buf,
            "create {} hash:net maxelem {}",
            temp_set,
            ips.len() + 10000
        )
        .unwrap();

        for ip in ips {
            if let IpNetwork::V4(ip) = ip {
                writeln!(&mut buf, "add {} {}", temp_set, ip).unwrap();
            }
        }

        writeln!(&mut buf, "swap {} {}", temp_set, self.name).unwrap();
        writeln!(&mut buf, "destroy {}", temp_set).unwrap();

        cmd!(&self.ipset_path, "restore").stdin_bytes(buf).run()?;

        Ok(())
    }
}
