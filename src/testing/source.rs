use crate::code::checksum::ChecksumTree;
use crate::code::sha256::make_checksum;
use crate::code::source::Language;
use crate::lambda::RouteKey;
use std::fs;
use std::path::PathBuf;

pub struct TestSource {
    #[allow(unused)]
    pub(crate) abs_path: PathBuf,
    #[allow(unused)]
    pub(crate) rel_path: PathBuf,
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

    pub(crate) fn build(
        self,
        project_name: &String,
        project_dir: &PathBuf,
        api_id: &String,
    ) -> TestSource {
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
