use std::fs::{self};

use clap::ArgMatches;
use log::debug;
use serde_json::Value;

use crate::{
    dirs,
    profile::{load_profiles, Profile},
};

pub fn execute(matches: &ArgMatches) {
    let name = matches.get_one::<String>("name").map(|v| v.to_owned());

    if let Some(name) = name {
        run_profile(name);
    } else {
        println!("no profile found");
    }
}

fn run_profile(name: String) {
    let profiles_dir = dirs::profiles_dir();
    if !profiles_dir.exists() {
        println!("no profiles available. {:#?}", profiles_dir);
        return;
    }

    let profiles = load_profiles(&profiles_dir).unwrap();

    println!("profiles:");
    let profiles = profiles
        .iter()
        .map(|p| {
            println!("  {:?}", p.name());
            return p;
        })
        .filter(|p| *p.name() == name)
        .collect::<Vec<&Profile>>();

    if profiles.len() != 1 {
        println!("no profile found");
        return;
    }

    let profile = profiles.first().unwrap();

    launch_profile(profile).unwrap();
}

fn launch_profile(profile: &Profile) -> anyhow::Result<()> {
    // launch the profile
    {
        let asset_index: String;
        let mut libraries: Vec<String> = Vec::new();

        let meta_path = dirs::meta_dir().join(format!("vanilla+{}.json", &"25w06a"));
        if !meta_path.exists() {
            // todo: better error handling
            panic!("meta does not exsist!");
        }

        let json: Value = serde_json::from_slice(&fs::read(meta_path)?)?;

        asset_index = json["assets"].as_str().unwrap().to_owned();
        for library in json["libraries"].as_array().unwrap() {
            libraries.push(
                dirs::libraries_dir()
                    .join(library["downloads"]["artifact"]["path"].as_str().unwrap())
                    .to_string_lossy()
                    .to_string(),
            );
        }

        let mut output = std::process::Command::new("java");
        output
            .arg("-cp")
            .arg(format!(
                "{};{}",
                profile
                    .path()
                    .join(".minecraft")
                    .join("client.jar")
                    .to_str()
                    .unwrap(),
                libraries.join(";"),
            ))
            .arg("net.minecraft.client.main.Main")
            .arg("--username")
            .arg("testxd")
            .arg("--version")
            .arg("")
            .arg("--gameDir")
            .arg(profile.path().join("minecraft"))
            .arg("-assetsDir")
            .arg(dirs::assets_dir())
            .arg("--assetIndex")
            .arg(asset_index.to_string())
            .arg("--uuid")
            .arg("--demo")
            .arg("--accessToken")
            .arg("--userType")
            .arg("--versionType")
            .arg("snapshot");

        println!("{}", format!("{:?}", output).replace("\"", ""));
        output.spawn().unwrap();
    }

    Ok(())
}



fn check_assets() {
    debug!("checking assets");


}
