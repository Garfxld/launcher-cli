use std::{
    fs::{self, File},
    str::FromStr,
    sync::Arc,
};

use clap::ArgMatches;
use futures::TryStreamExt;

use crate::{
    dirs,
    profile::{normalize_name, Profile},
    ModLoader,
};

pub async fn execute(matches: &ArgMatches) {
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
        let profile_dir = dirs::profiles_dir().join(normalize_name(&name));
        if replace && profile_dir.exists() {
            fs::remove_dir_all(profile_dir).unwrap();
        }
    }

    let profile = Profile::create(name, crate::version::GameVersion {}).unwrap();

    download_meta().await.unwrap();
    download_assets().await.unwrap();
    download_libraries().await.unwrap();
    download_client(&profile).await.unwrap();
}

async fn download_meta() -> anyhow::Result<()> {
    let manifest_dir = dirs::meta_dir();
    if !manifest_dir.exists() {
        fs::create_dir_all(&manifest_dir)?;
    }

    let manifest_path = manifest_dir.join("vanilla+25w06a.json");
    if manifest_path.exists() {
        return Ok(());
    }

    let bytes = reqwest::get("https://piston-meta.mojang.com/v1/packages/019cd0c018635c33a4acdc7320adf010bd5e66ae/25w06a.json").await?.text().await?;
    let file = File::create(&manifest_path)?;
    serde_json::to_writer_pretty(
        &file,
        &serde_json::from_str::<serde_json::Value>(&bytes).unwrap(),
    )?;

    Ok(())
}

async fn download_assets() -> anyhow::Result<()> {
    let manifest_path = dirs::meta_dir().join("vanilla+25w06a.json");
    if !manifest_path.exists() {
        anyhow::bail!("error"); // todo: return error
    }

    let bytes = fs::read(&manifest_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    // indexes
    let indexes_dir = dirs::assets_dir().join("indexes");
    if !indexes_dir.exists() {
        fs::create_dir_all(&indexes_dir)?;
    }

    let index_path = indexes_dir.join(format!(
        "{}.json",
        json["assetIndex"]["id"].as_str().unwrap()
    ));
    if !index_path.exists() {
        let bytes = reqwest::get(json["assetIndex"]["url"].as_str().unwrap())
            .await?
            .text()
            .await?;
        let file = File::create(&index_path)?;
        serde_json::to_writer_pretty(
            &file,
            &serde_json::from_str::<serde_json::Value>(&bytes).unwrap(),
        )?;
    }

    // objects
    let objects_dir = dirs::assets_dir().join("objects");
    if !objects_dir.exists() {
        fs::create_dir_all(&objects_dir)?;
    }

    let bytes = fs::read(&index_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    use futures::{stream, StreamExt};

    let hashes = json["objects"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(_, v)| v["hash"].as_str().unwrap())
        .map(|hash| (hash[0..2].to_string(), hash.to_string()))
        .collect::<Vec<(String, String)>>();

    let objects_dir = Arc::new(&objects_dir);
    stream::iter(hashes)
        .map(Ok)
        .try_for_each_concurrent(8, |(a, b)| {
            let objects_dir = Arc::clone(&objects_dir);
            async move {
                let object_subdir = objects_dir.join(&a);
                if !object_subdir.exists() {
                    fs::create_dir_all(&object_subdir)?;
                }

                let object_path = object_subdir.join(&b);
                if !object_path.exists() {
                    let url = format!("https://resources.download.minecraft.net/{}/{}", a, b);
                    let data = reqwest::get(&url).await?.bytes().await?;
                    fs::write(&object_path, &data).unwrap();
                }
                anyhow::Ok(())
            }
        })
        .await?;

    Ok(())
}

async fn download_libraries() -> anyhow::Result<()> {
    let manifest_path = dirs::meta_dir().join("vanilla+25w06a.json");
    if !manifest_path.exists() {
        anyhow::bail!("error"); // todo: return error
    }

    let bytes = fs::read(&manifest_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    for library in json["libraries"].as_array().unwrap() {
        let library_path =
            dirs::libraries_dir().join(library["downloads"]["artifact"]["path"].as_str().unwrap());

        if library_path.exists() {
            continue;
        }

        fs::create_dir_all(library_path.parent().unwrap())?;
        let data = reqwest::get(library["downloads"]["artifact"]["url"].as_str().unwrap())
            .await?
            .bytes()
            .await?;
        fs::write(&library_path, &data)?;
    }

    Ok(())
}

async fn download_client(profile: &Profile) -> anyhow::Result<()> {
    let manifest_path = dirs::meta_dir().join("vanilla+25w06a.json");
    if !manifest_path.exists() {
        anyhow::bail!("error"); // todo: return error
    }

    let bytes = fs::read(&manifest_path)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    let run_dir = profile.path().join(".minecraft");
    if !run_dir.exists() {
        fs::create_dir_all(&run_dir)?;
    }

    let client_path = run_dir.join("client.jar");
    if client_path.exists() {
        return Ok(());
    }

    let bytes = reqwest::get(json["downloads"]["client"]["url"].as_str().unwrap())
        .await?
        .bytes()
        .await?;
    File::create(&client_path)?;
    fs::write(&client_path, &bytes)?;

    Ok(())
}
