use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_iam::primitives::DateTime;
use aws_sdk_iam::types::Role;
use temp_dir::TempDir;

use crate::aws::clients::AwsClients;
use crate::aws::{AwsApiDeets, AwsDeets};
use crate::code::build::BuildMode;
use crate::code::checksum::ChecksumTree;
use crate::code::parse::parse_source_file;
use crate::code::runtime::SourcesRuntimeDeets;
use crate::code::sha256::make_checksum;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, SourceFile};
use crate::lambda::RouteKey;
use crate::project::Lx3ProjectDeets;

pub struct ProjectTest {
    pub api_id: String,
    pub project_deets: Arc<Lx3ProjectDeets>,
    pub project_dir: PathBuf,
    pub project_name: String,
    #[allow(unused)]
    temp_dir: TempDir,
    #[allow(unused)]
    sources: Vec<TestSource>,
}

impl ProjectTest {
    pub fn builder() -> ProjectTestBuilder {
        ProjectTestBuilder::new()
    }

    pub fn with_file(path: &str, content: &str) -> Self {
        ProjectTest::builder()
            .with_source(TestSource::with_path(path).content(content))
            .build()
    }

    pub fn lambda_checksum_path(&self, route_key: &RouteKey) -> PathBuf {
        ChecksumTree::dir_path(
            &self.project_dir,
            &self.api_id,
            &route_key.to_fn_name(&self.project_name),
        )
    }

    pub fn path(&self, path: &str) -> PathBuf {
        self.project_dir.join(path)
    }

    pub fn source_file(&self, path: &str) -> SourceFile {
        debug_assert!(&self.project_dir.join(path).is_file());
        parse_source_file(self.source_path(path), &Default::default()).unwrap()
    }

    pub fn source_path(&self, path: &str) -> SourcePath {
        SourcePath::from_rel(&self.project_dir, PathBuf::from(path))
    }

    pub fn source_paths(&self) -> Vec<SourcePath> {
        self.sources
            .iter()
            .map(|s| self.source_path(s.rel_path.to_string_lossy().as_ref()))
            .collect()
    }
}

pub struct ProjectTestBuilder {
    api_id: Option<String>,
    build_mode: Option<BuildMode>,
    project_name: Option<String>,
    sources: Vec<TestSourceBuilder>,
}

impl ProjectTestBuilder {
    fn new() -> Self {
        Self {
            api_id: None,
            build_mode: None,
            project_name: None,
            sources: Vec::new(),
        }
    }

    pub fn build(self) -> ProjectTest {
        let temp_dir = TempDir::new().unwrap();
        let api_id = self.api_id.unwrap_or("API_ID".to_string());
        let project_name = self.project_name.unwrap_or("PROJECT_NAME".to_string());
        let project_dir = temp_dir.path().to_path_buf();
        let sources = self
            .sources
            .into_iter()
            .map(|s| s.build(&project_name, &project_dir, &api_id))
            .collect();
        let project_deets = Arc::new(
            Lx3ProjectDeets::builder()
                .aws_deets(AwsDeets {
                    account_id: "account_id".to_string(),
                    api: AwsApiDeets {
                        id: api_id.clone(),
                        stage_name: "development".to_string(),
                    },
                    region: Region::new("us-east-1"),
                    sdk_clients: AwsClients::from(
                        &SdkConfig::builder()
                            .behavior_version(BehaviorVersion::v2024_03_28())
                            .build(),
                    ),
                    lambda_role: Role::builder()
                        .arn("arn")
                        .create_date(DateTime::from_secs(1))
                        .path("path")
                        .role_id("role_id")
                        .role_name("role_name")
                        .build()
                        .unwrap(),
                })
                .build_mode(self.build_mode.unwrap_or(BuildMode::Debug))
                .runtime_deets(SourcesRuntimeDeets::default())
                .build(project_dir.clone(), project_name.clone()),
        );
        ProjectTest {
            api_id,
            project_deets,
            project_name,
            temp_dir,
            project_dir,
            sources,
        }
    }

    pub fn api_id(mut self, api_id: &str) -> Self {
        self.api_id = Some(api_id.to_string());
        self
    }

    pub fn build_mode(mut self, build_mode: BuildMode) -> Self {
        self.build_mode = Some(build_mode);
        self
    }

    pub fn project_name(mut self, project_name: &str) -> Self {
        self.project_name = Some(project_name.to_string());
        self
    }

    pub fn with_source(mut self, source: TestSourceBuilder) -> Self {
        self.sources.push(source);
        self
    }
}

pub struct TestSource {
    #[allow(unused)]
    abs_path: PathBuf,
    #[allow(unused)]
    rel_path: PathBuf,
}

impl TestSource {
    pub fn with_path(path: &str) -> TestSourceBuilder {
        TestSourceBuilder::new(PathBuf::from(path), None)
    }

    fn with_route_key(file_name: String, route_key: RouteKey) -> TestSourceBuilder {
        TestSourceBuilder::new(
            route_key.to_route_dir_path().join(file_name),
            Some(route_key),
        )
    }

    pub fn http_fn(language: Language, route_key: RouteKey) -> TestSourceBuilder {
        Self::with_route_key(
            format!(
                "lambda.{}",
                match language {
                    Language::JavaScript => "js",
                    Language::Python => "py",
                    Language::TypeScript => "ts",
                }
            ),
            route_key,
        )
    }

    pub fn method_env_var(route_key: RouteKey) -> TestSourceBuilder {
        Self::with_route_key(
            format!(
                "lambda.{}.env",
                route_key.http_method.to_string().to_lowercase()
            ),
            route_key,
        )
    }

    pub fn path_env_var(route_key: RouteKey) -> TestSourceBuilder {
        Self::with_route_key("lambda.env".to_string(), route_key)
    }
}

enum TestSourceChecksum {
    Clean(Option<RouteKey>),
    Dirty(Option<RouteKey>),
    None,
}

pub struct TestSourceBuilder {
    checksum: TestSourceChecksum,
    content: Option<String>,
    path: PathBuf,
    route_key: Option<RouteKey>,
}

impl TestSourceBuilder {
    fn new(path: PathBuf, route_key: Option<RouteKey>) -> Self {
        assert!(path.is_relative());
        TestSourceBuilder {
            checksum: TestSourceChecksum::None,
            content: None,
            path,
            route_key,
        }
    }

    fn build(self, project_name: &String, project_dir: &PathBuf, api_id: &String) -> TestSource {
        let abs_path = project_dir.join(&self.path);
        let rel_path = self.path;
        fs::create_dir_all(abs_path.parent().unwrap()).unwrap();
        fs::write(&abs_path, self.content.unwrap_or("".to_string())).unwrap();
        match match self.checksum {
            TestSourceChecksum::Clean(fn_name) => {
                Some((make_checksum(&abs_path).unwrap(), fn_name))
            }
            TestSourceChecksum::Dirty(fn_name) => Some(("dirty".to_string(), fn_name)),
            TestSourceChecksum::None => None,
        } {
            None => {}
            Some((checksum, fn_name)) => {
                assert!(fn_name.is_some() || self.route_key.is_some());
                let checksum_path = ChecksumTree::dir_path(
                    project_dir,
                    api_id,
                    &fn_name.or(self.route_key).unwrap().to_fn_name(project_name),
                )
                .join(&rel_path);
                fs::create_dir_all(checksum_path.parent().unwrap()).unwrap();
                fs::write(checksum_path, checksum).unwrap();
            }
        }
        TestSource { abs_path, rel_path }
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn with_clean_checksum(mut self) -> Self {
        assert!(self.route_key.is_some());
        self.checksum = TestSourceChecksum::Clean(None);
        self
    }

    pub fn with_clean_checksum_for_fn(mut self, route_key: RouteKey) -> Self {
        self.checksum = TestSourceChecksum::Clean(Some(route_key));
        self
    }

    pub fn with_dirty_checksum(mut self) -> Self {
        assert!(self.route_key.is_some());
        self.checksum = TestSourceChecksum::Dirty(None);
        self
    }

    pub fn with_dirty_checksum_for_fn(mut self, route_key: RouteKey) -> Self {
        self.checksum = TestSourceChecksum::Dirty(Some(route_key));
        self
    }

    pub fn without_checksum(mut self) -> Self {
        self.checksum = TestSourceChecksum::None;
        self
    }
}
