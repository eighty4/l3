use std::io::Write;
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
    let data_dir = PathBuf::from(".l3");
    let routes_dir = PathBuf::from("routes");
    if config_path.exists() || data_dir.exists() || routes_dir.exists() {
        println!("error: current directory already contains an l3 project");
        exit(1);
    }
    println!("creating project `{}` in current directory", dir_name);

    let gitignore = PathBuf::from(".gitignore");
    if gitignore.exists() {
        let mut f = fs::OpenOptions::new().append(true).open(gitignore)?;
        f.write_all(".l3\n".as_bytes())?;
        println!("✔ added .l3 data directory to .gitignore");
    } else {
        fs::write(gitignore, ".l3\n")?;
        println!("✔ created .gitignore");
    }

    fs::write(config_path, format!("project_name: {}\n", dir_name))?;
    println!("✔ created l3.yml config");

    fs::create_dir("routes")?;
    println!("✔ created routes directory for HTTP lambdas");

    fs::write(routes_dir.join("get.mjs"), include_str!("[gen]get.mjs"))?;
    println!("✔ created routes/get.mjs");

    Ok(())
}
