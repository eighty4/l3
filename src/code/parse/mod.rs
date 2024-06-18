use std::collections::HashMap;
use std::path::Path;

use anyhow::anyhow;

use crate::lambda::HttpMethod;

mod es_module;

pub fn parse_module_for_lambda_handlers(
    path: &Path,
) -> Result<HashMap<HttpMethod, String>, anyhow::Error> {
    let file_extension = path.extension().unwrap().to_string_lossy().to_string();
    let exported_fns = if file_extension == "js" || file_extension == "mjs" {
        es_module::parse_module_for_exported_fns(path)
    } else {
        return Err(anyhow!(
            "{file_extension} is not a supported file type for source file {}",
            path.to_string_lossy()
        ));
    };

    let mut lambda_fns = HashMap::new();
    for exported_fn in exported_fns {
        if let Ok(http_method) = HttpMethod::try_from(exported_fn.as_str()) {
            if lambda_fns.contains_key(&http_method) {
                return Err(anyhow!(
                    "multiple {http_method} functions found in source file {}",
                    path.to_string_lossy()
                ));
            }
            lambda_fns.insert(http_method, exported_fn);
        }
    }

    Ok(lambda_fns)
}
