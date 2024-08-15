use crate::code::runtime::node::{read_node_config, NodeConfig};
use crate::code::runtime::typescript::{read_typescript_config, TypeScriptConfig};
use crate::code::runtime::RuntimeConfigMessage::*;
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
                _ = completed.send(JavaScript);
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
                _ = completed.send(TypeScript);
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
            let (tx, mut rx) = oneshot::channel();
            self.msg_tx
                .send(RuntimeConfigMessage::RefreshConfig {
                    completed: Some(tx),
                    language,
                })
                .unwrap();
            _ = rx.await;
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

#[derive(Clone, Default)]
pub struct RuntimeConfig {
    // todo import_resolvers: HashMap<Language, Arc<Box<dyn ImportResolver>>>,
    node_config: Arc<NodeConfig>,
    // todo source_parsers: HashMap<Language, Arc<Box<dyn SourceParser>>>,
    typescript_config: Arc<TypeScriptConfig>,
}

impl RuntimeConfig {
    pub fn new(project_dir: PathBuf) -> (Arc<Mutex<RuntimeConfig>>, Arc<RuntimeConfigApi>) {
        let (msg_tx, msg_rx) = unbounded_channel();
        let runtime_config: Arc<Mutex<RuntimeConfig>> = Default::default();
        let mut event_loop =
            RuntimeConfigEventLoop::new(msg_rx, Arc::new(project_dir), runtime_config.clone());
        tokio::spawn(async move { event_loop.start().await });
        (runtime_config, RuntimeConfigApi::new(msg_tx))
    }

    pub fn node_config(&self) -> Arc<NodeConfig> {
        self.node_config.clone()
    }

    pub fn set_node_config(&mut self, node_config: NodeConfig) {
        self.node_config = Arc::new(node_config);
    }

    pub fn set_typescript_config(&mut self, typescript_config: TypeScriptConfig) {
        self.typescript_config = Arc::new(typescript_config);
    }

    pub fn typescript_config(&self) -> Arc<TypeScriptConfig> {
        self.typescript_config.clone()
    }
}
