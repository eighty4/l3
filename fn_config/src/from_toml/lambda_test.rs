use std::collections::HashMap;
use std::path::PathBuf;

use crate::configs::*;

#[test]
fn test_parse_lambda_toml() {
    let s = r#"
name = "my-data-fn"
source = "./lambdas/data.ts"
handler = "doDataThings"
env_files = [".env.data"]
env_vars = ["DATA_DATA_DATA=data!"]
node_version = 22
"#;
    assert_eq!(
        LambdaConfig::try_from(&s.parse::<toml::Table>().unwrap()),
        Ok(LambdaConfig {
            name: Some("my-data-fn".into()),
            source: Some("./lambdas/data.ts".into()),
            handler: Some("doDataThings".into()),
            env: Some(LambdaEnvConfig {
                env_files: Some(vec![PathBuf::from(".env.data").into()]),
                env_vars: Some(HashMap::from([("DATA_DATA_DATA".into(), "data!".into())])),
            }),
            runtime: Some(LambdaRuntimeConfig {
                node: Some(NodeLambdaConfig {
                    version: Some(NodeVersion::TwentyTwo),
                }),
                python: None
            }),
        })
    );
}
