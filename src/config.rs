use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use yaml_rust::{Yaml, YamlLoader};

pub struct Config {
    pub project_name: Option<String>,
}

pub fn project_name() -> Result<Option<String>, anyhow::Error> {
    Ok(read_config()?.and_then(|c| c.project_name))
}

pub fn read_config() -> Result<Option<Config>, anyhow::Error> {
    let contents = match fs::read_to_string("l3.yml") {
        Ok(contents) => contents,
        Err(_) => return Err(anyhow!("l3.yml does not exist")),
    };
    let docs = match YamlLoader::load_from_str(&contents) {
        Ok(docs) => docs,
        Err(err) => return Err(anyhow!("yaml error parsing l3.yml: {err}")),
    };

    let mut project_name: Option<String> = None;

    for doc in docs {
        if let Yaml::Hash(h) = doc {
            for (k, v) in h {
                if k.as_str().unwrap() == "project_name" {
                    project_name = v.as_str().map(String::from)
                }
            }
        }
    }

    if project_name.is_some() {
        Ok(Some(Config { project_name }))
    } else {
        Ok(None)
    }
}

pub fn read_api_id_from_data_dir() -> Result<Option<String>, anyhow::Error> {
    let p = PathBuf::from(".l3/api");
    if p.exists() {
        match fs::read_to_string(p) {
            Ok(api_id) => Ok(Some(api_id.trim().to_string())),
            Err(err) => Err(anyhow!("error reading --api_id from .l3/api: {err}")),
        }
    } else {
        Ok(None)
    }
}

pub fn write_api_id_to_data_dir(api_id: &String) -> Result<(), anyhow::Error> {
    match fs::create_dir_all(format!(".l3/{api_id}")).and_then(|_| fs::write(".l3/api", api_id)) {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("error writing --api_id to .l3/api: {err}")),
    }
}
