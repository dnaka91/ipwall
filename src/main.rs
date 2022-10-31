#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

use anyhow::Result;
use clap::Parser;
use ipwall::{
    firewall::{Firewall, IpSet},
    provider::{Provider, Source},
    settings::{IptablesTarget, Settings},
    state::State,
    LastModified,
};

#[derive(Parser)]
#[command(about, author, version)]
struct Opts {
    #[arg(short, long)]
    uninstall: bool,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let settings = Settings::load()?;
    let mut state = State::load()?;

    if settings.firehol.level1 {
        run_safe(
            &Source::FireHolLevel1,
            settings.target,
            opts.uninstall,
            &mut state,
        );
    }
    if settings.firehol.level2 {
        run_safe(
            &Source::FireHolLevel2,
            settings.target,
            opts.uninstall,
            &mut state,
        );
    }
    if settings.firehol.level3 {
        run_safe(
            &Source::FireHolLevel3,
            settings.target,
            opts.uninstall,
            &mut state,
        );
    }

    for (name, url) in settings.sources {
        run_safe(
            &Source::Custom { name, url },
            settings.target,
            opts.uninstall,
            &mut state,
        );
    }

    Ok(())
}

fn run_safe(source: &Source, target: IptablesTarget, uninstall: bool, state: &mut State) {
    let command = if uninstall { "uninstall" } else { "install" };

    match run(source, target, uninstall, state) {
        Ok(lm) => {
            println!("{}ed {} filters", command, source.name());

            if let Some(lm) = lm {
                state.last_modified.insert(source.name().to_owned(), lm);

                if let Err(e) = state.save() {
                    eprintln!("error saving settings:\n{:?}", e);
                }
            }
        }

        Err(e) => eprintln!("error {}ing {} filters:\n{:?}", command, source.name(), e),
    }
}

fn run(
    source: &Source,
    target: IptablesTarget,
    uninstall: bool,
    state: &State,
) -> Result<LastModified> {
    let firewall = IpSet::new(source.name(), target)?;

    if uninstall {
        firewall.uninstall()?;
        return Ok(None);
    }

    firewall.install()?;

    let (ips, last_modified) =
        Provider::new(source).get(state.last_modified.get(source.name()).copied())?;

    if !ips.is_empty() {
        firewall.block(&ips)?;
    }

    Ok(last_modified)
}
