use std::path::PathBuf;

pub fn root_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("launcher")
}

pub fn assets_dir() -> PathBuf {
    root_dir().join("assets")
}

pub fn libraries_dir() -> PathBuf {
    root_dir().join("libraries")
}

pub fn meta_dir() -> PathBuf {
    root_dir().join("meta")
}

pub fn profiles_dir() -> PathBuf {
    root_dir().join("profiles")
}