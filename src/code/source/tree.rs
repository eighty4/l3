use crate::code::read::recursively_read_dirs;
use crate::code::source::path::SourcePath;
use crate::code::source::tree::SourceTreeMessage::*;
use crate::code::source::{Language, ModuleImport, ModuleImports, SourceFile};
use crate::lambda::{LambdaFn, RouteKey};
use crate::project::Lx3Project;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

// todo stream FileUpdate to SourceTreeMessage::ProcessSourcePath
enum SourceTreeMessage {
    ProcessSourceFile {
        processed: Option<oneshot::Sender<bool>>,
        source_file: SourceFile,
    },
    ProcessSourcePath {
        processed: Option<oneshot::Sender<bool>>,
        source_path: SourcePath,
    },
}

struct SourceTreeEventLoop {
    msg_rx: UnboundedReceiver<SourceTreeMessage>,
    msg_tx: UnboundedSender<SourceTreeMessage>,
    project: Arc<Lx3Project>,
    source_tree: Arc<Mutex<SourceTree>>,
}

impl SourceTreeEventLoop {
    fn new(
        msg_rx: UnboundedReceiver<SourceTreeMessage>,
        msg_tx: UnboundedSender<SourceTreeMessage>,
        project: Arc<Lx3Project>,
        source_tree: Arc<Mutex<SourceTree>>,
    ) -> Self {
        Self {
            msg_rx,
            msg_tx,
            project,
            source_tree,
        }
    }

    async fn start(&mut self) {
        loop {
            tokio::select! {
                source_msg_opt = self.msg_rx.recv() => {
                    self.handle_message(source_msg_opt.unwrap());
                }
            }
        }
    }

    fn handle_message(&mut self, msg: SourceTreeMessage) {
        match msg {
            ProcessSourceFile {
                source_file,
                processed,
            } => self.process_source_file(source_file, processed),
            ProcessSourcePath {
                source_path,
                processed,
            } => self.process_source_path(source_path, processed),
        }
    }

    fn process_source_file(
        &self,
        source_file: SourceFile,
        processed: Option<oneshot::Sender<bool>>,
    ) {
        let msg_tx = self.msg_tx.clone();
        let project = self.project.clone();
        let source_tree = self.source_tree.clone();
        tokio::spawn(async move {
            // todo prevent circular imports cause infinite queueing
            let import_resolver = {
                project
                    .runtime_config
                    .lock()
                    .unwrap()
                    .import_resolver(&source_file.language)
            };
            let mut processing_imports: Vec<oneshot::Receiver<bool>> = Vec::new();
            let mut source_file = source_file;
            if let ModuleImports::Unprocessed(imports) = &source_file.imports {
                let mut processed_imports: Vec<ModuleImport> = Vec::new();
                for import_specifier in imports {
                    let import = import_resolver.resolve(&source_file.path, import_specifier);
                    match &import {
                        ModuleImport::PackageDependency { .. } => todo!(),
                        ModuleImport::RelativeSource(source_path) => {
                            let (tx, rx) = oneshot::channel();
                            processing_imports.push(rx);
                            msg_tx.send(ProcessSourcePath {
                                source_path: source_path.clone(),
                                processed: Some(tx),
                            })?;
                        }
                        ModuleImport::Unknown(_) => todo!(),
                    };
                    processed_imports.push(import);
                }
                source_file.imports = ModuleImports::Processed(processed_imports);
            }

            let lambda_fns = if source_file.path.rel.starts_with("routes") {
                // todo send LambdaNotification src warning if routes dir src without lambda handler
                source_file.collect_lambda_fns(&project)
            } else {
                None
            };
            {
                let mut source_tree = source_tree.lock().unwrap();
                source_tree.add_source_file(source_file);
                if let Some(lambda_fns) = lambda_fns {
                    for lambda_fn in lambda_fns {
                        source_tree.add_lambda_fn(lambda_fn);
                    }
                }
            }
            for completed in processing_imports {
                completed.await?;
            }
            if let Some(processed) = processed {
                processed.send(true).unwrap();
            }
            Ok::<(), anyhow::Error>(())
        });
    }

    fn process_source_path(
        &self,
        source_path: SourcePath,
        processed: Option<oneshot::Sender<bool>>,
    ) {
        let msg_tx = self.msg_tx.clone();
        let source_parser = {
            self.project
                .runtime_config
                .lock()
                .unwrap()
                .source_parser(&Language::from_extension(&source_path.rel).unwrap())
        };
        tokio::spawn(async move {
            // todo surface error with LambdaNotification
            match source_parser.parse(source_path) {
                Ok(source_file) => msg_tx.send(ProcessSourceFile {
                    source_file,
                    processed,
                })?,
                Err(err) => panic!("{err}"),
            }
            Ok::<(), anyhow::Error>(())
        });
    }
}

pub struct SourcesApi {
    msg_tx: UnboundedSender<SourceTreeMessage>,
    project: Arc<Lx3Project>,
}

impl SourcesApi {
    fn new(msg_tx: UnboundedSender<SourceTreeMessage>, project: Arc<Lx3Project>) -> Arc<Self> {
        Arc::new(Self { msg_tx, project })
    }

    pub async fn refresh_routes(&self) -> Result<(), anyhow::Error> {
        let mut processing: Vec<oneshot::Receiver<bool>> = Vec::new();
        for path in recursively_read_dirs(&self.project.dir.join("routes")).await? {
            if SourcePath::is_lambda_file_name(&path) {
                let (sender, receiver) = oneshot::channel();
                processing.push(receiver);
                self.msg_tx.send(ProcessSourcePath {
                    source_path: SourcePath::from_abs(&self.project.dir, path),
                    processed: Some(sender),
                })?;
            }
        }
        for completed in processing {
            completed.await?;
        }
        Ok(())
    }
}

pub struct SourceTree {
    lambdas: HashMap<RouteKey, Arc<LambdaFn>>,
    /// SourceFile instances mapped by relative path
    sources: HashMap<PathBuf, Arc<SourceFile>>,
}

impl SourceTree {
    pub fn new(project: Arc<Lx3Project>) -> (Arc<Mutex<Self>>, Arc<SourcesApi>) {
        let (msg_tx, msg_rx) = unbounded_channel();
        let source_tree = Arc::new(Mutex::new(SourceTree {
            lambdas: HashMap::new(),
            sources: HashMap::new(),
        }));
        let mut event_loop =
            SourceTreeEventLoop::new(msg_rx, msg_tx.clone(), project.clone(), source_tree.clone());
        tokio::spawn(async move { event_loop.start().await });
        (source_tree, SourcesApi::new(msg_tx, project))
    }

    fn add_lambda_fn(&mut self, lambda_fn: Arc<LambdaFn>) {
        self.lambdas.insert(lambda_fn.route_key.clone(), lambda_fn);
    }

    fn add_source_file(&mut self, source_file: SourceFile) {
        self.sources
            .insert(source_file.path.rel.clone(), Arc::new(source_file));
    }

    pub fn lambda_fns(&mut self) -> Vec<Arc<LambdaFn>> {
        self.lambdas.values().cloned().collect()
    }

    pub fn lambda_fn_by_route_key(&mut self, route_key: &RouteKey) -> Option<Arc<LambdaFn>> {
        self.lambdas.get(route_key).cloned()
    }

    #[allow(unused)]
    pub fn get_source_file(&self, path: &Path) -> Option<Arc<SourceFile>> {
        debug_assert!(path.is_relative());
        self.sources.get(path).cloned()
    }
}
