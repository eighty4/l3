use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

pub struct InitOptions {
    pub project_name: String,
}

pub(crate) fn init_project(init_opts: InitOptions) -> Result<(), anyhow::Error> {
    let config_path = PathBuf::from("l3.yml");
    let data_dir = PathBuf::from(".l3");
    let routes_dir = PathBuf::from("routes");
    if config_path.exists() || data_dir.exists() || routes_dir.exists() {
        println!("error: current directory already contains an l3 project");
        exit(1);
    }
    println!(
        "creating project `{}` in current directory",
        init_opts.project_name
    );

    let gitignore = PathBuf::from(".gitignore");
    if gitignore.exists() {
        let mut f = fs::OpenOptions::new().append(true).open(gitignore)?;
        f.write_all(".l3\n".as_bytes())?;
        println!("✔ added .l3 data directory to .gitignore");
    } else {
        fs::write(gitignore, ".l3\n")?;
        println!("✔ created .gitignore");
    }

    fs::write(
        config_path,
        format!("project_name: {}\n", init_opts.project_name),
    )?;
    println!("✔ created l3.yml config");

    fs::create_dir("routes")?;
    println!("✔ created routes directory for HTTP lambdas");

    fs::write(routes_dir.join("get.mjs"), include_str!("[gen]get.mjs"))?;
    println!("✔ created routes/get.mjs");

    Ok(())
}
