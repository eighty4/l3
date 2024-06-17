use std::path::PathBuf;
use std::{env, fmt};

#[derive(Clone)]
pub enum HttpMethod {
    Delete,
    Get,
    Patch,
    Post,
    Put,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
        }
    }
}

pub struct LambdaFn {
    pub name: String,
    method: HttpMethod,
    path: PathBuf,
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

    pub fn api_method(&self) -> HttpMethod {
        self.method.clone()
    }

    pub fn api_path(&self) -> String {
        self.file_path()
            .trim_start_matches("routes/")
            .trim_end_matches(self.path.file_name().unwrap().to_str().unwrap())
            .trim_end_matches('/')
            .to_string()
    }

    pub fn file_path(&self) -> String {
        self.path
            .strip_prefix(env::current_dir().unwrap())
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
