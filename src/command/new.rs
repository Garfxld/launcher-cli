use std::{
    fs::{self, File},
    path::PathBuf,
    str::FromStr,
};

use clap::ArgMatches;

use crate::{
    command::run::profiles_dir,
    profile::{self, normalize_name, Profile},
    root_dir, ModLoader,
};

pub fn execute(matches: &ArgMatches) {
    let name = matches.get_one::<String>("name").map(|v| v.to_owned());
    let version = matches
        .get_one::<String>("version")
        .map(|v| v.to_owned())
        .unwrap();
    let loader = matches.get_one::<String>("loader").map(|v| v.to_owned());
    let replace = *matches.get_one::<bool>("replace").unwrap_or(&false); // todo: remove

    let name = name.unwrap_or(version.clone());
    let loader = ModLoader::from_str(loader.unwrap_or("".to_owned()).as_str()).ok();

    println!("name:    {}", &name);
    println!("version: {}", &version);
    println!(
        "loader:  {}",
        &loader
            .map(|l| l.to_string())
            .unwrap_or("vanilla".to_string())
    );

    {
        let profile_dir = profiles_dir().join(normalize_name(&name));
        if replace && profile_dir.exists() {
            fs::remove_dir_all(profile_dir).unwrap();
        }
    }

    let profile = Profile::create(name).unwrap();

    download_meta().unwrap();
    download_assets().unwrap();
    download_libraries().unwrap();
    download_client(&profile).unwrap();
}

fn download_meta() -> anyhow::Result<()> {
    let manifest_dir = manifest_dir();
    if !manifest_dir.exists() {
        fs::create_dir_all(&manifest_dir)?;
    }

    let manifest_path = manifest_dir.join("vanilla+25w05a.json");
    if manifest_path.exists() {
        return Ok(());
    }

    let bytes = reqwest::blocking::get("https://piston-meta.mojang.com/v1/packages/af26a4b3605f891007f08000846909840e80784a/25w05a.json")?.text()?;
    let file = File::create(&manifest_path)?;
    serde_json::to_writer_pretty(
        &file,
        &serde_json::from_str::<serde_json::Value>(&bytes).unwrap(),
    )?;

    Ok(())
}

fn manifest_dir() -> PathBuf {
    root_dir().join("meta")
}

fn download_assets() -> anyhow::Result<()> {
    let manifest_path = manifest_dir().join("vanilla+25w05a.json");
    if !manifest_path.exists() {
        anyhow::bail!("error"); // todo: return error
    }

    let bytes = fs::read(&manifest_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    // indexes
    let indexes_dir = assets_dir().join("indexes");
    if !indexes_dir.exists() {
        fs::create_dir_all(&indexes_dir)?;
    }

    let index_path = indexes_dir.join(format!(
        "{}.json",
        json["assetIndex"]["id"].as_str().unwrap()
    ));
    if !index_path.exists() {
        let bytes = reqwest::blocking::get(json["assetIndex"]["url"].as_str().unwrap())?.text()?;
        let file = File::create(&index_path)?;
        serde_json::to_writer_pretty(
            &file,
            &serde_json::from_str::<serde_json::Value>(&bytes).unwrap(),
        )?;
    }

    // objects
    let objects_dir = assets_dir().join("objects");
    if !objects_dir.exists() {
        fs::create_dir_all(&objects_dir)?;
    }

    let bytes = fs::read(&index_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    for (_, value) in json["objects"].as_object().unwrap() {
        let hash = value["hash"].as_str().unwrap();
        let object_subdir = objects_dir.join(hash[0..2].to_string());
        if !object_subdir.exists() {
            fs::create_dir_all(&object_subdir)?;
        }

        let object_path = object_subdir.join(hash);
        if !object_path.exists() {
            let bytes = reqwest::blocking::get(format!(
                "https://resources.download.minecraft.net/{}/{}",
                hash[0..2].to_string(),
                hash
            ))?
            .bytes()?;
            File::create(&object_path)?;
            fs::write(&object_path, &bytes)?;
        }
    }

    Ok(())
}

fn assets_dir() -> PathBuf {
    root_dir().join("assets")
}


fn download_libraries() -> anyhow::Result<()> {
    let manifest_path = manifest_dir().join("vanilla+25w05a.json");
    if !manifest_path.exists() {
        anyhow::bail!("error"); // todo: return error
    }

    let bytes = fs::read(&manifest_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    for library in json["libraries"].as_array().unwrap() {
        let library_path = libraries_dir().join(library["downloads"]["artifact"]["path"].as_str().unwrap());

        if library_path.exists() {
            continue;
        }

        fs::create_dir_all(library_path.parent().unwrap())?;
        File::create(&library_path)?;
        let bytes = reqwest::blocking::get(library["downloads"]["artifact"]["url"].as_str().unwrap())?.bytes()?;
        fs::write(&library_path, bytes)?;
    }


    Ok(())
}

fn libraries_dir() -> PathBuf {
    root_dir().join("libraries")
}


fn download_client(profile: &Profile) -> anyhow::Result<()> {
    let manifest_path = manifest_dir().join("vanilla+25w05a.json");
    if !manifest_path.exists() {
        anyhow::bail!("error"); // todo: return error
    }

    let bytes = fs::read(&manifest_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    let run_dir = profile.path().join("minecraft");
    if !run_dir.exists() {
        fs::create_dir_all(&run_dir)?;
    }

    let client_path = run_dir.join("client.jar");
    if client_path.exists() {
        return Ok(());
    }

    let bytes = reqwest::blocking::get(json["downloads"]["client"]["url"].as_str().unwrap())?.bytes()?;
    File::create(&client_path)?;
    fs::write(&client_path, &bytes)?;

    Ok(())
}