use std::fs;

use yaml_rust::{Yaml, YamlLoader};

pub struct Config {
    pub project_name: Option<String>,
}

pub fn project_name() -> Result<Option<String>, anyhow::Error> {
    Ok(read_config()?.and_then(|c| c.project_name))
}

pub fn read_config() -> Result<Option<Config>, anyhow::Error> {
    let contents = fs::read_to_string("l3.yml")?;
    let docs = YamlLoader::load_from_str(&contents)?;

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
