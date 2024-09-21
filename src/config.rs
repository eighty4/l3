use anyhow::anyhow;
use std::fs;
use std::path::Path;
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

pub fn is_valid_project_name(project_name: &str) -> bool {
    if project_name.len() < 3 || project_name.len() > 16 {
        return false;
    }
    let mut chars = project_name.chars();
    if !chars.next().map(|c| c.is_alphabetic()).unwrap() {
        return false;
    }
    for c in chars {
        let valid = c.is_alphanumeric() || c == '-' || c == '_';
        if !valid {
            return false;
        }
    }
    !project_name
        .chars()
        .last()
        .map(|c| c == '-' || c == '_')
        .unwrap()
}

// todo rewrite invalid dir name into a valid project name
pub fn suggested_project_name_from_directory(p: &Path) -> String {
    debug_assert!(p.is_absolute());
    debug_assert!(p.is_dir());
    p.file_name().unwrap().to_string_lossy().to_string()
}
