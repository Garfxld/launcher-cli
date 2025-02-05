use std::{fs, path::PathBuf, str::FromStr};

use clap::{arg, Arg, Command};

mod command;

mod dirs;
mod profile;
mod version;

fn main() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");

    env_logger::init_from_env(env);

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main().await })
}

async fn async_main() {
    #[rustfmt::skip]
    let matches = Command::new("launcher")
        .subcommands([
            Command::new("new")
                .about("Create a new profile")
                .args(&[
                    Arg::new("name").short('n').long("name")
                        .num_args(1),
                    Arg::new("version").short('v').long("version")
                        .required(true)
                        .num_args(1),
                    Arg::new("loader").short('l').long("loader")
                        .num_args(1),
                    arg!(-r --replace "debug: deletes the profile if it already exists and creates a new one").num_args(0)
                ]),
            Command::new("run")
                .alias("r")
                .about("Run a profile")
                .arg(
                    Arg::new("name")
                        .help("the name of the profile to run")
                        .required(true)
                        .num_args(1),
                ),
        ])
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand() {
        Some(("new", new_matches)) => {
            command::new::execute(&new_matches).await;
        }
        Some(("run", run_matches)) => {
            command::run::execute(&run_matches);
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone)]
pub enum ModLoader {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl FromStr for ModLoader {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fabric" => Ok(ModLoader::Fabric),
            "quilt" => Ok(ModLoader::Quilt),
            "forge" => Ok(ModLoader::Forge),
            "neoforge" => Ok(ModLoader::NeoForge),
            _ => Err(()),
        }
    }
}

impl ToString for ModLoader {
    fn to_string(&self) -> String {
        match self {
            ModLoader::Fabric => "fabric",
            ModLoader::Quilt => "quilt",
            ModLoader::Forge => "forge",
            ModLoader::NeoForge => "neoforge",
        }
        .to_owned()
    }
}

pub enum Environment {
    Client,
    Server,
}
