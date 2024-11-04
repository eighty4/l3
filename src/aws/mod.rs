use anyhow::anyhow;
use aws_sdk_iam::types::Role;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::aws::clients::AwsClients;
use crate::aws::preflight::AwsPreflightData;
use crate::aws::resources::repository::AwsResources;

pub(crate) mod clients;
pub(crate) mod preflight;
pub(crate) mod resources;
pub(crate) mod tasks;

pub const DEFAULT_STAGE_NAME: &str = "development";

/// Program inputs specifying AWS Api Gateway resources and names.
pub struct AwsApiGatewayConfig {
    pub api_id: Option<String>,
    pub stage_name: Option<String>,
}

/// API Gateway id and Stage identities.
pub struct AwsApiGateway {
    pub id: String,
    pub stage_name: String,
}

impl AwsApiGateway {
    pub fn new(id: String, stage_name: String) -> Arc<Self> {
        Arc::new(Self { id, stage_name })
    }
}

pub struct AwsProject {
    pub account_id: String,
    pub api: Arc<AwsApiGateway>,
    pub lambda_role: Role,
    pub resources: Arc<AwsResources>,
    pub sdk_clients: Arc<AwsClients>,
}

impl From<AwsPreflightData> for AwsProject {
    fn from(v: AwsPreflightData) -> Self {
        Self {
            account_id: v.account_id,
            api: v.api,
            lambda_role: v.lambda_role,
            resources: v.resources,
            sdk_clients: v.sdk_clients,
        }
    }
}

pub struct AwsDataDir {}

impl AwsDataDir {
    pub fn clear_cache(api_id: &String, project_dir: &Path) {
        let _ = fs::remove_dir_all(Self::path(project_dir).join(api_id));
    }

    fn path(project_dir: &Path) -> PathBuf {
        project_dir.join(".l3/aws")
    }

    pub fn read_cached_api_id(project_dir: &Path) -> Result<Option<String>, anyhow::Error> {
        let p = Self::path(project_dir).join("api");
        if p.exists() {
            match fs::read_to_string(p) {
                Ok(api_id) => Ok(Some(api_id.trim().to_string())),
                Err(err) => Err(anyhow!("error reading --api_id from .l3/aws/api: {err}")),
            }
        } else {
            Ok(None)
        }
    }

    pub fn cache_api_id(project_dir: &Path, api_id: &String) -> Result<(), anyhow::Error> {
        match fs::create_dir_all(Self::path(project_dir).join(api_id))
            .and_then(|_| fs::write(Self::path(project_dir).join("api"), api_id))
        {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!("error writing --api_id to .l3/aws/api: {err}")),
        }
    }
}
