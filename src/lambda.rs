use std::env;
use std::path::PathBuf;

pub enum HttpMethod {
    Delete,
    Get,
    Patch,
    Post,
    Put,
}

pub struct LambdaFn {
    pub name: String,
    #[allow(unused)]
    pub method: HttpMethod,
    #[allow(unused)]
    pub path: PathBuf,
}

impl LambdaFn {
    pub fn new(project_name: &String, method: HttpMethod, path: PathBuf) -> Self {
        let fn_label = path
            .strip_prefix(env::current_dir().unwrap())
            .unwrap()
            .to_string_lossy()
            .to_string()
            .replace(['/', '\\'], "-")
            .trim_end_matches(".js")
            .trim_end_matches(".mjs")
            .to_string();
        let name = format!("l3-{}-{}-fn", project_name, fn_label);
        Self { name, method, path }
    }
}
