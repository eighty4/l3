use std::collections::HashMap;
use std::path::Path;

use anyhow::anyhow;

use crate::code::parse::es_module::EsModule;
use crate::lambda::HttpMethod;

mod es_module;

#[cfg(test)]
mod es_module_test;
#[cfg(test)]
mod parse_test;
#[cfg(test)]
mod ts_module_test;

pub fn parse_module_for_lambda_handlers(
    path: &Path,
) -> Result<HashMap<HttpMethod, String>, anyhow::Error> {
    if !path.is_file() {
        return Err(anyhow!(
            "source file does not exist at {}",
            path.to_string_lossy()
        ));
    }
    let file_extension = match path.extension() {
        None => {
            return Err(anyhow!(
                "file extension missing for source file {}",
                path.to_string_lossy()
            ))
        }
        Some(ext) => ext.to_string_lossy().to_string(),
    };
    let exported_fns = if file_extension == "js" || file_extension == "mjs" {
        EsModule::parse(path)?.exported_fns
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
