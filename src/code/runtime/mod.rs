use crate::code::parse::imports::node::NodeImportResolver;
use crate::code::parse::imports::typescript::TypescriptImportResolver;
use crate::code::parse::imports::ImportResolver;
use crate::code::parse::swc::SwcSourceParser;
use crate::code::parse::SourceParser;
use crate::code::runtime::node::{read_node_config, NodeConfig};
use crate::code::runtime::typescript::{read_typescript_config, TypeScriptConfig};
use crate::code::runtime::RuntimeConfigMessage::*;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, Language::*};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

pub(crate) mod node;
pub(crate) mod python;
pub(crate) mod typescript;

#[cfg(test)]
mod node_test;
#[cfg(test)]
mod typescript_test;

enum RuntimeConfigMessage {
    RefreshConfig {
        completed: Option<oneshot::Sender<Language>>,
        language: Language,
    },
}

struct RuntimeConfigEventLoop {
    msg_rx: UnboundedReceiver<RuntimeConfigMessage>,
    project_dir: Arc<PathBuf>,
    runtime_config: Arc<Mutex<RuntimeConfig>>,
}

impl RuntimeConfigEventLoop {
    fn new(
        msg_rx: UnboundedReceiver<RuntimeConfigMessage>,
        project_dir: Arc<PathBuf>,
        runtime_config: Arc<Mutex<RuntimeConfig>>,
    ) -> Self {
        Self {
            msg_rx,
            project_dir,
            runtime_config,
        }
    }

    async fn start(&mut self) {
        loop {
            if let Some(msg) = self.msg_rx.recv().await {
                match msg {
                    RefreshConfig {
                        language,
                        completed,
                    } => match language {
                        JavaScript => self.refresh_node_config(completed),
                        Python => todo!(),
                        TypeScript => self.refresh_typescript_config(completed),
                    },
                }
            }
        }
    }

    fn refresh_node_config(&mut self, completed: Option<oneshot::Sender<Language>>) {
        let project_dir = self.project_dir.clone();
        let runtime_config = self.runtime_config.clone();
        tokio::spawn(async move {
            let node_config = read_node_config(project_dir.as_ref());
            runtime_config.lock().unwrap().set_node_config(node_config);
            if let Some(completed) = completed {
                let _ = completed.send(JavaScript);
            }
        });
    }

    fn refresh_typescript_config(&mut self, completed: Option<oneshot::Sender<Language>>) {
        let project_dir = self.project_dir.clone();
        let runtime_config = self.runtime_config.clone();
        tokio::spawn(async move {
            let typescript_config = read_typescript_config(project_dir.as_ref());
            runtime_config
                .lock()
                .unwrap()
                .set_typescript_config(typescript_config);
            if let Some(completed) = completed {
                let _ = completed.send(TypeScript);
            }
        });
    }
}

pub struct RuntimeConfigApi {
    msg_tx: UnboundedSender<RuntimeConfigMessage>,
}

impl RuntimeConfigApi {
    fn new(msg_tx: UnboundedSender<RuntimeConfigMessage>) -> Arc<Self> {
        Arc::new(Self { msg_tx })
    }

    pub async fn initialize_runtime_configs(&self) {
        for language in [JavaScript, TypeScript] {
            let (tx, rx) = oneshot::channel();
            self.msg_tx
                .send(RuntimeConfigMessage::RefreshConfig {
                    completed: Some(tx),
                    language,
                })
                .unwrap();
            let _ = rx.await;
        }
    }

    pub fn refresh_node_config(&self) {
        self.refresh_config(JavaScript);
    }

    pub fn refresh_typescript_config(&self) {
        self.refresh_config(TypeScript);
    }

    fn refresh_config(&self, language: Language) {
        self.msg_tx
            .send(RuntimeConfigMessage::RefreshConfig {
                completed: None,
                language,
            })
            .unwrap();
    }
}

#[derive(Clone)]
pub struct RuntimeConfig {
    javascript_source_parser: Arc<Box<dyn SourceParser>>,
    node_config: Arc<NodeConfig>,
    node_import_resolver: Arc<Box<dyn ImportResolver>>,
    project_dir: Arc<PathBuf>,
    typescript_config: Arc<TypeScriptConfig>,
    typescript_import_resolver: Arc<Box<dyn ImportResolver>>,
    typescript_source_parser: Arc<Box<dyn SourceParser>>,
}

impl RuntimeConfig {
    pub fn new(project_dir: PathBuf) -> (Arc<Mutex<RuntimeConfig>>, Arc<RuntimeConfigApi>) {
        let project_dir = Arc::new(project_dir);
        let (msg_tx, msg_rx) = unbounded_channel();
        let node_config: Arc<NodeConfig> = Default::default();
        let javascript_source_parser: Arc<Box<dyn SourceParser>> =
            Arc::new(Box::new(SwcSourceParser::for_javascript()));
        let node_import_resolver: Arc<Box<dyn ImportResolver>> =
            Arc::new(Box::new(NodeImportResolver::new(node_config.clone())));
        let typescript_config: Arc<TypeScriptConfig> = Default::default();
        let typescript_import_resolver: Arc<Box<dyn ImportResolver>> =
            Arc::new(Box::new(TypescriptImportResolver::new()));
        let typescript_source_parser: Arc<Box<dyn SourceParser>> =
            Arc::new(Box::new(SwcSourceParser::for_typescript()));
        let runtime_config: Arc<Mutex<RuntimeConfig>> = Arc::new(Mutex::new(RuntimeConfig {
            javascript_source_parser,
            node_config,
            node_import_resolver,
            project_dir: project_dir.clone(),
            typescript_config,
            typescript_import_resolver,
            typescript_source_parser,
        }));
        let mut event_loop =
            RuntimeConfigEventLoop::new(msg_rx, project_dir, runtime_config.clone());
        tokio::spawn(async move { event_loop.start().await });
        (runtime_config, RuntimeConfigApi::new(msg_tx))
    }

    pub fn import_resolver(&self, language: &Language) -> Arc<Box<dyn ImportResolver>> {
        match language {
            JavaScript => self.node_import_resolver.clone(),
            Python => todo!(),
            TypeScript => self.typescript_import_resolver.clone(),
        }
    }

    #[allow(unused)]
    pub fn get_node_config(&self) -> Arc<NodeConfig> {
        self.node_config.clone()
    }

    /// Additional sources to include in lambda archives
    pub fn runtime_sources(&self, language: &Language) -> Vec<SourcePath> {
        match language {
            JavaScript | TypeScript => vec![SourcePath::from_rel(
                self.project_dir.as_ref(),
                PathBuf::from("package.json"),
            )],
            _ => Vec::new(),
        }
    }

    pub fn set_node_config(&mut self, node_config: NodeConfig) {
        self.node_config = Arc::new(node_config);
        self.node_import_resolver =
            Arc::new(Box::new(NodeImportResolver::new(self.node_config.clone())));
        self.typescript_import_resolver = Arc::new(Box::new(TypescriptImportResolver::new()));
    }

    pub fn set_typescript_config(&mut self, typescript_config: TypeScriptConfig) {
        self.typescript_config = Arc::new(typescript_config);
    }

    pub fn source_parser(&self, language: &Language) -> Arc<Box<dyn SourceParser>> {
        match language {
            JavaScript => self.javascript_source_parser.clone(),
            Python => todo!(),
            TypeScript => self.typescript_source_parser.clone(),
        }
    }

    #[allow(unused)]
    pub fn get_typescript_config(&self) -> Arc<TypeScriptConfig> {
        self.typescript_config.clone()
    }
}
