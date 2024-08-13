use crate::code::env::EnvVarSources;
use crate::code::parse::parse_source_file;
use crate::code::read::parallel::recursively_read_dirs;
use crate::code::source::path::SourcePath;
use crate::code::source::tree::SourceTreeMessage::*;
use crate::code::source::SourceFile;
use crate::lambda::{LambdaFn, RouteKey};
use crate::notification::LambdaNotification;
use crate::project::Lx3ProjectDeets;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

enum SourceTreeMessage {
    ProcessSourcePath {
        source_path: SourcePath,
        processed: Option<oneshot::Sender<bool>>,
    },
    ProcessSourceFile {
        source_file: SourceFile,
        processed: Option<oneshot::Sender<bool>>,
    },
}

struct SourceTreeEventLoop {
    msg_rx: UnboundedReceiver<SourceTreeMessage>,
    msg_tx: UnboundedSender<SourceTreeMessage>,
    notification_tx: UnboundedSender<LambdaNotification>,
    project_deets: Arc<Lx3ProjectDeets>,
    source_tree: Arc<Mutex<SourceTree>>,
}

impl SourceTreeEventLoop {
    fn new(
        msg_rx: UnboundedReceiver<SourceTreeMessage>,
        msg_tx: UnboundedSender<SourceTreeMessage>,
        notification_tx: UnboundedSender<LambdaNotification>,
        project_deets: Arc<Lx3ProjectDeets>,
        source_tree: Arc<Mutex<SourceTree>>,
    ) -> Self {
        Self {
            msg_rx,
            msg_tx,
            notification_tx,
            project_deets,
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
            ProcessSourcePath {
                source_path,
                processed,
            } => self.process_source_path(source_path, processed),
            ProcessSourceFile {
                source_file,
                processed,
            } => self.process_source_file(source_file, processed),
        }
    }

    fn process_source_path(
        &self,
        source_path: SourcePath,
        processed: Option<oneshot::Sender<bool>>,
    ) {
        let msg_tx = self.msg_tx.clone();
        let project_deets = self.project_deets.clone();
        tokio::spawn(async move {
            // todo surface error with LambdaNotification
            match parse_source_file(source_path, &project_deets.sources) {
                Ok(source_file) => msg_tx.send(ProcessSourceFile {
                    source_file,
                    processed,
                })?,
                Err(err) => panic!("{err}"),
            }
            Ok::<(), anyhow::Error>(())
        });
    }

    fn process_source_file(
        &self,
        source_file: SourceFile,
        processed: Option<oneshot::Sender<bool>>,
    ) {
        let project_deets = self.project_deets.clone();
        let source_tree = self.source_tree.clone();
        tokio::spawn(async move {
            let mut lambda_fns = Vec::new();
            if source_file.path.rel.starts_with("routes") {
                let handler_fns = source_file.collect_http_handler_fn_names();
                if handler_fns.is_empty() {
                    // todo send LambdaNotification
                } else {
                    let http_path = RouteKey::extract_http_path(&source_file.path.rel).unwrap();
                    // todo move to SourceFile?
                    for (http_method, handler_fn) in handler_fns {
                        let route_key = RouteKey::new(http_method, http_path.clone());
                        let env_var_sources =
                            EnvVarSources::new(&project_deets.project_dir, &route_key).unwrap();
                        lambda_fns.push(LambdaFn::new(
                            env_var_sources,
                            handler_fn,
                            source_file.path.clone(),
                            project_deets.clone(),
                            route_key,
                        ));
                    }
                }
            }
            // todo source_file.imports
            {
                let mut source_tree = source_tree.lock().unwrap();
                source_tree.add_source_file(source_file);
                for lambda_fn in lambda_fns {
                    source_tree.add_lambda_fn(lambda_fn);
                }
            }
            if let Some(processed) = processed {
                processed.send(true).unwrap();
            }
        });
    }
}

pub struct SourcesApi {
    msg_tx: UnboundedSender<SourceTreeMessage>,
    project_deets: Arc<Lx3ProjectDeets>,
}

impl SourcesApi {
    fn new(
        msg_tx: UnboundedSender<SourceTreeMessage>,
        project_deets: Arc<Lx3ProjectDeets>,
    ) -> Arc<Self> {
        Arc::new(Self {
            msg_tx,
            project_deets,
        })
    }

    pub async fn refresh_routes(&self) -> Result<(), anyhow::Error> {
        let mut processing: Vec<oneshot::Receiver<bool>> = Vec::new();
        for path in recursively_read_dirs(&self.project_deets.project_dir.join("routes")).await? {
            if SourcePath::is_lambda_file_name(&path) {
                let (sender, receiver) = oneshot::channel();
                processing.push(receiver);
                self.msg_tx.send(ProcessSourcePath {
                    source_path: SourcePath::from_abs(&self.project_deets.project_dir, path),
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
    project_deets: Arc<Lx3ProjectDeets>,
    sources: Vec<Arc<SourceFile>>,
}

impl SourceTree {
    pub fn new(
        notification_tx: UnboundedSender<LambdaNotification>,
        project_deets: Arc<Lx3ProjectDeets>,
    ) -> (Arc<Mutex<Self>>, Arc<SourcesApi>) {
        let (msg_tx, msg_rx) = unbounded_channel();
        let source_tree = Arc::new(Mutex::new(SourceTree {
            lambdas: HashMap::new(),
            project_deets: project_deets.clone(),
            sources: Vec::new(),
        }));
        let mut event_loop = SourceTreeEventLoop::new(
            msg_rx,
            msg_tx.clone(),
            notification_tx,
            project_deets.clone(),
            source_tree.clone(),
        );
        tokio::spawn(async move { event_loop.start().await });
        let sources_api = SourcesApi::new(msg_tx, project_deets);
        (source_tree, sources_api)
    }

    fn add_lambda_fn(&mut self, lambda_fn: LambdaFn) {
        self.lambdas
            .insert(lambda_fn.route_key.clone(), Arc::new(lambda_fn));
    }

    fn add_source_file(&mut self, source_file: SourceFile) {
        self.sources.push(Arc::new(source_file));
    }

    pub fn lambda_fns(&mut self) -> Vec<Arc<LambdaFn>> {
        self.lambdas.values().cloned().collect()
    }

    pub fn lambda_fn_by_route_key(&mut self, route_key: &RouteKey) -> Option<Arc<LambdaFn>> {
        self.lambdas.get(route_key).cloned()
    }
}
