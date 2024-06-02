use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};

pub(crate) fn init_project() -> Result<(), anyhow::Error> {
    let dir_name = env::current_dir()?
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let config_path = PathBuf::from("l3.yml");
    let routes_dir = PathBuf::from("routes");
    if config_path.exists() || routes_dir.exists() {
        println!("error: current directory already contains an l3 project");
        exit(1);
    }
    println!("creating project `{}` in current directory", dir_name);
    fs::write(config_path, format!("project_name: {}\n", dir_name))?;
    println!("✔ created ./l3.yml config");
    fs::create_dir("routes")?;
    println!("✔ created ./routes directory for HTTP lambdas");
    fs::write(routes_dir.join("get.mjs"), include_str!("[gen]get.mjs"))?;
    println!("✔ created ./routes/get.mjs");
    Ok(())
}
