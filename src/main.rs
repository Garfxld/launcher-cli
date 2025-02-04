use std::{fs, path::PathBuf, str::FromStr};

use clap::{arg, Arg, Command};

mod command;

mod profile;
mod version;

fn main() {
    #[rustfmt::skip]
    let matches = Command::new("launcher")
        .subcommands([
            Command::new("new")
                .about("Create a new profile")
                .args(&[
                    Arg::new("name").short('N').long("name")
                        .num_args(1),
                    Arg::new("version").short('V').long("version")
                        .required(true)
                        .num_args(1),
                    Arg::new("loader").short('L').long("loader")
                        .num_args(1),
                    arg!(-R --replace "debug: deletes the profile if it already exists and creates a new one").num_args(0)
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
            command::new::execute(&new_matches);
        }
        Some(("run", run_matches)) => {
            command::run::execute(&run_matches);
        }
        _ => unreachable!(),
    }
}

fn root_dir() -> PathBuf {
    let home = dirs::data_dir().unwrap();

    let data_dir = home.join("launcher");
    fs::create_dir_all(&data_dir).unwrap();

    data_dir
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
        match s {
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
