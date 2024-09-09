use std::fs;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use aws_config::Region;
use aws_sdk_iam::types::Role;

use crate::aws::clients::AwsClients;
use crate::aws::preflight::AwsPreflightData;

pub(crate) mod clients;
mod config;
mod fetch;
pub(crate) mod lambda;
pub(crate) mod preflight;
pub(crate) mod state;
pub(crate) mod tasks;

#[cfg(test)]
mod lambda_test;

#[cfg(test)]
mod state_test;

pub const DEFAULT_STAGE_NAME: &str = "development";

/// Program inputs specifying AWS Api Gateway resources and names.
pub struct AwsApiConfig {
    pub api_id: Option<String>,
    pub stage_name: Option<String>,
}

/// API Gateway API and Stage identities.
pub struct AwsApiDeets {
    pub id: String,
    pub stage_name: String,
}

pub struct AwsDeets {
    pub account_id: String,
    pub api: AwsApiDeets,
    pub lambda_role: Role,
    pub sdk_clients: AwsClients,
    pub region: Region,
}

impl From<AwsPreflightData> for AwsDeets {
    fn from(v: AwsPreflightData) -> Self {
        Self {
            account_id: v.account_id,
            api: v.api,
            lambda_role: v.lambda_role,
            region: v.region,
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
