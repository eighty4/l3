use crate::aws::clients::AwsClients;
use crate::aws::resources::repository::AwsResources;
use crate::aws::{AwsApiGateway, AwsProject};
use crate::code::build::BuildMode;
use crate::code::checksum::ChecksumTree;
use crate::code::runtime::RuntimeConfig;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, SourceFile};
use crate::lambda::RouteKey;
use crate::project::Lx3Project;
use crate::testing::files::recursively_copy_dir;
use crate::testing::source::{TestSource, TestSourceBuilder};
use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_apigatewayv2::primitives::DateTime;
use aws_sdk_iam::types::Role;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use temp_dir::TempDir;

pub struct ProjectTest {
    pub api_id: String,
    pub project: Arc<Lx3Project>,
    pub project_dir: PathBuf,
    pub project_name: String,
    #[allow(unused)]
    temp_dir: TempDir,
    #[allow(unused)]
    sources: Vec<TestSource>,
    verify_with_runtime: bool,
    write_project_sources: Option<String>,
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

    /// Creates absolute path to Lambda source directory
    pub fn lambda_route_dir(&self, route_key: &RouteKey) -> PathBuf {
        self.project_dir.join(route_key.to_route_dir_path())
    }

    pub fn path(&self, path: &str) -> PathBuf {
        self.project_dir.join(path)
    }

    pub fn parse_result(&self, path: &str) -> Result<SourceFile, anyhow::Error> {
        debug_assert!(&self.project_dir.join(path).is_file());
        self.project
            .runtime_config
            .lock()
            .unwrap()
            .source_parser(&Language::from_extension(&PathBuf::from(&path)).unwrap())
            .parse(self.source_path(path))
    }

    pub fn _runtime_config(&self) -> Arc<Mutex<RuntimeConfig>> {
        self.project.runtime_config.clone()
    }

    pub fn source_file(&self, path: &str) -> SourceFile {
        self.parse_result(path).unwrap()
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

    pub fn verify_runtime(&self) -> Result<(), anyhow::Error> {
        for source in &self.sources {
            if source.rel_path.starts_with("routes") {
                match Language::from_extension(&source.rel_path) {
                    Some(Language::JavaScript) => {
                        let completed_command = Command::new("node")
                            .arg(&source.abs_path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()?
                            .wait_with_output()?;
                        if completed_command.status.code().unwrap() != 0 {
                            println!(
                                "!!! `node {}`\n!!! stdout:\n{}\n!!! stderr:\n{}",
                                &source.abs_path.to_string_lossy(),
                                String::from_utf8_lossy(completed_command.stdout.as_slice()),
                                String::from_utf8_lossy(completed_command.stderr.as_slice()),
                            );
                            panic!();
                        }
                    }
                    _ => todo!(),
                }
            }
        }
        Ok(())
    }

    fn write_project_sources(&self, test_dir: &str) {
        let out_dir = PathBuf::from("target").join("test").join(test_dir);
        let _ = fs::remove_dir(&out_dir);
        let _ = fs::create_dir_all(&out_dir);
        for source in &self.sources {
            let out_file = out_dir.join(&source.rel_path);
            let _ = fs::create_dir_all(out_file.parent().unwrap());
            fs::copy(&source.abs_path, out_file).unwrap();
        }
        recursively_copy_dir(self.project_dir.join(".l3"), out_dir.join(".l3")).unwrap();
    }
}

impl Drop for ProjectTest {
    fn drop(&mut self) {
        if self.verify_with_runtime {
            self.verify_runtime().unwrap();
        }
        if let Some(test_dir) = &self.write_project_sources {
            self.write_project_sources(test_dir.as_str());
        }
    }
}

pub struct ProjectTestBuilder {
    api_id: Option<String>,
    build_mode: Option<BuildMode>,
    project_name: Option<String>,
    sources: Vec<TestSourceBuilder>,
    verify_with_runtime: bool,
    write_project_sources: Option<String>,
}

impl ProjectTestBuilder {
    fn new() -> Self {
        Self {
            api_id: None,
            build_mode: None,
            project_name: None,
            sources: Vec::new(),
            verify_with_runtime: false,
            write_project_sources: None,
        }
    }

    pub fn build(self) -> ProjectTest {
        let temp_dir = TempDir::new().unwrap();
        let api_id = self.api_id.unwrap_or("API_ID".to_string());
        let project_name = self.project_name.unwrap_or("PROJECT_NAME".to_string());
        let project_dir = temp_dir.path().canonicalize().unwrap();
        let sources = self
            .sources
            .into_iter()
            .map(|s| s.build(&project_name, &project_dir, &api_id))
            .collect();
        let (runtime_config, _) = RuntimeConfig::new(project_dir.clone());
        let api = AwsApiGateway::new(api_id.clone(), "development".to_string());
        let sdk_clients = Arc::new(AwsClients::from(
            SdkConfig::builder()
                .behavior_version(BehaviorVersion::v2024_03_28())
                .region(Region::new("us-east-1"))
                .build(),
        ));
        let resources = AwsResources::new(api.clone(), project_name.clone(), sdk_clients.clone());
        let (project, _notification_rx) = Lx3Project::builder()
            .aws(Arc::new(AwsProject {
                account_id: "account_id".to_string(),
                api,
                resources,
                sdk_clients,
                lambda_role: Role::builder()
                    .arn("arn")
                    .create_date(DateTime::from_secs(1))
                    .path("path")
                    .role_id("role_id")
                    .role_name("role_name")
                    .build()
                    .unwrap(),
            }))
            .build_mode(self.build_mode.unwrap_or(BuildMode::Debug))
            .runtime_config(runtime_config)
            .build(project_dir.clone(), project_name.clone());
        ProjectTest {
            api_id,
            project,
            project_name,
            temp_dir,
            project_dir,
            sources,
            verify_with_runtime: self.verify_with_runtime,
            write_project_sources: self.write_project_sources,
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

    pub fn verify_with_runtime(mut self) -> Self {
        self.verify_with_runtime = true;
        self
    }

    pub fn with_source(mut self, source: TestSourceBuilder) -> Self {
        self.sources.push(source);
        self
    }

    pub fn with_sources(mut self, mut sources: Vec<TestSourceBuilder>) -> Self {
        self.sources.append(&mut sources);
        self
    }

    pub fn write_project_sources(mut self, test_dir: &str) -> Self {
        self.write_project_sources = Some(String::from(test_dir));
        self
    }
}
