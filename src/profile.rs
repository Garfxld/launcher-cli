use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    str::FromStr,
};


use crate::{dirs, version::GameVersion, ModLoader};

#[derive(Debug)]
pub struct Profile {
    name: String,
    slug: String,
    path: PathBuf,
    loader: Option<ModLoader>,
}


impl Profile {

    pub fn create<S>(name: S, version: GameVersion) -> anyhow::Result<Self>
    where
        S: Into<String>,
    {
        let name = name.into();
        let slug = normalize_name(&name);

        let profiles_dir = dirs::profiles_dir();
        if !profiles_dir.exists() {
            fs::create_dir(&profiles_dir)?;
        }
        let path = profiles_dir.join(&slug);

        if path.exists() && fs::read_dir(&path)?.next().is_some() {
            return Err(ProfileError::AlreadyExists.into());
        }
        if !path.exists() {
            fs::create_dir(&path)?;
        }

        let mod_loader: Option<ModLoader> = None;

        let file = File::create(path.join("profile.json"))?;
        serde_json::to_writer_pretty(
            file,
            &serde_json::json!({
                "name": name,
                "slug": slug,
                "type": mod_loader.clone().map(|s| s.to_string()).unwrap_or("vanilla".to_string()),
                "version": "25w05a",
            }),
        )?;

        Ok(Self {
            name,
            slug,
            path,
            loader: mod_loader,
        })
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let json: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("profile.json"))?)?;

        Ok(Self {
            name: json["name"].as_str().unwrap().into(),
            slug: json["slug"].as_str().unwrap().into(),
            path: path.to_path_buf(),
            loader: ModLoader::from_str(json["type"].as_str().unwrap()).ok(),
        })
    }

    pub fn delete(self) -> anyhow::Result<()> {
        fs::remove_dir_all(self.path)?;
        Ok(())
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn slug(&self) -> &String {
        &self.slug
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn loader(&self) -> &Option<ModLoader> {
        &self.loader
    }
}

pub fn normalize_name(name: &String) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

pub fn load_profiles(path: &Path) -> anyhow::Result<Vec<Profile>> {
    let mut profiles = Vec::new();
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        profiles.push(Profile::load(&path)?);
    }

    Ok(profiles)
}






#[derive(thiserror::Error, Debug)]
pub enum ProfileError {
    #[error("Profile already exists")]
    AlreadyExists,
}
