use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};

pub struct InitOptions {
    pub project_name: Option<String>,
}

pub(crate) fn init_project(opts: InitOptions) -> Result<(), anyhow::Error> {
    let project_name = match opts.project_name {
        Some(project_name) => project_name,
        None => project_name_from_current_dir()?,
    };
    let config_path = PathBuf::from("l3.yml");
    let data_dir = PathBuf::from(".l3");
    let routes_dir = PathBuf::from("routes");
    if config_path.exists() || data_dir.exists() || routes_dir.exists() {
        println!("error: current directory already contains an l3 project");
        exit(1);
    }
    println!("creating project `{}` in current directory", project_name);

    let gitignore = PathBuf::from(".gitignore");
    if gitignore.exists() {
        let mut f = fs::OpenOptions::new().append(true).open(gitignore)?;
        f.write_all(".l3\n".as_bytes())?;
        println!("✔ added .l3 data directory to .gitignore");
    } else {
        fs::write(gitignore, ".l3\n")?;
        println!("✔ created .gitignore");
    }

    fs::write(config_path, format!("project_name: {}\n", project_name))?;
    println!("✔ created l3.yml config");

    fs::create_dir("routes")?;
    println!("✔ created routes directory for HTTP lambdas");

    fs::write(routes_dir.join("get.mjs"), include_str!("[gen]get.mjs"))?;
    println!("✔ created routes/get.mjs");

    Ok(())
}

fn project_name_from_current_dir() -> Result<String, anyhow::Error> {
    Ok(env::current_dir()?
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string())
}
