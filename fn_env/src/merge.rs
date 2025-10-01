use std::collections::HashMap;

#[allow(unused)]
type EnvVarsCollection = Vec<Option<HashMap<String, String>>>;

/// Merge env vars ordered from lowest to highest precedence
#[allow(unused)]
pub fn merge_env_vars(env_vars: EnvVarsCollection) -> HashMap<String, String> {
    let env_vars = env_vars
        .into_iter()
        .filter(|x| x.is_some())
        .collect::<EnvVarsCollection>();
    match env_vars.len() {
        0 => HashMap::new(),
        1 => env_vars.first().unwrap().to_owned().unwrap_or_default(),
        _ => {
            let mut result = HashMap::new();
            for env_vars in env_vars.into_iter().flatten() {
                for (k, v) in env_vars {
                    result.insert(k, v);
                }
            }
            result
        }
    }
}
