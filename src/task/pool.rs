use crate::lambda::RouteKey;
use crate::task::executor::TaskExecutor;
use crate::task::queue::BuildQueue;
use crate::task::{LambdaTask, LambdaTaskKind};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

struct TaskPoolEventLoop {
    lambda_task_rx: UnboundedReceiver<LambdaTask>,
    queues: HashMap<RouteKey, BuildQueue>,
    task_executor: Arc<TaskExecutor>,
}

impl TaskPoolEventLoop {
    fn new(
        task_executor: Arc<TaskExecutor>,
        lambda_task_rx: UnboundedReceiver<LambdaTask>,
    ) -> Self {
        Self {
            task_executor,
            lambda_task_rx,
            queues: HashMap::new(),
        }
    }

    async fn start(&mut self) {
        loop {
            tokio::select! {
                lambda_task_opt = self.lambda_task_rx.recv() => {
                    if let Some(lambda_task) = lambda_task_opt {
                        self.handle_lambda_task(lambda_task);
                    }
                }
            }
        }
    }

    fn handle_lambda_task(&mut self, lambda_task: LambdaTask) {
        match self.queues.get_mut(&lambda_task.route_key) {
            None => {
                let mut queue = BuildQueue::new(self.task_executor.clone());
                let route_key = lambda_task.route_key.clone();
                queue.enqueue(lambda_task);
                self.queues.insert(route_key, queue);
            }
            Some(queue) => {
                queue.enqueue(lambda_task);
            }
        };
    }
}

pub struct TaskPool {
    lambda_task_tx: UnboundedSender<LambdaTask>,
}

impl TaskPool {
    pub fn new(task_executor: TaskExecutor) -> Arc<Self> {
        let (lambda_task_tx, lambda_task_rx) = unbounded_channel();
        let mut event_loop = TaskPoolEventLoop::new(Arc::new(task_executor), lambda_task_rx);
        tokio::spawn(async move { event_loop.start().await });
        Arc::new(Self { lambda_task_tx })
    }

    #[allow(unused)]
    pub fn lambda_task(
        &self,
        kind: LambdaTaskKind,
        route_key: RouteKey,
    ) -> Result<(), anyhow::Error> {
        self.lambda_task_tx.send(LambdaTask {
            completed: None,
            kind,
            route_key,
        })?;
        Ok(())
    }

    pub fn lambda_task_with_reply(
        &self,
        kind: LambdaTaskKind,
        route_key: RouteKey,
    ) -> Result<oneshot::Receiver<()>, anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        self.lambda_task_tx.send(LambdaTask {
            completed: Some(tx),
            kind,
            route_key,
        })?;
        Ok(rx)
    }
}
