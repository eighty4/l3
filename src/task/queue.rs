use crate::task::executor::TaskExecutor;
use crate::task::LambdaTask;
use crate::ui::exit::err_exit;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::{select, spawn};

enum TaskQueueMessage {
    Dequeue,
    Enqueue(LambdaTask),
}

struct BuildQueueEventLoop {
    build: Option<JoinHandle<()>>,
    task_executor: Arc<TaskExecutor>,
    msg_rx: UnboundedReceiver<TaskQueueMessage>,
    msg_tx: UnboundedSender<TaskQueueMessage>,
    queue: VecDeque<LambdaTask>,
}

impl BuildQueueEventLoop {
    fn new(
        task_executor: Arc<TaskExecutor>,
        msg_rx: UnboundedReceiver<TaskQueueMessage>,
        msg_tx: UnboundedSender<TaskQueueMessage>,
    ) -> Self {
        Self {
            build: None,
            task_executor,
            msg_rx,
            msg_tx,
            queue: VecDeque::new(),
        }
    }

    async fn start(&mut self) {
        loop {
            select! {
                queue_msg_opt = self.msg_rx.recv() => {
                    if let Some(queue_msg) = queue_msg_opt {
                        self.handle_queue_msg(queue_msg);
                    }
                }
            }
        }
    }

    pub fn handle_queue_msg(&mut self, queue_msg: TaskQueueMessage) {
        match queue_msg {
            TaskQueueMessage::Dequeue => self.try_next(),
            TaskQueueMessage::Enqueue(lambda_task) => self.enqueue(lambda_task),
        }
    }

    pub fn enqueue(&mut self, msg: LambdaTask) {
        if self.build.is_none() {
            self.now(msg);
        } else {
            self.queue.push_back(msg);
        }
    }

    pub fn now(&mut self, lambda_task: LambdaTask) {
        let task_executor = self.task_executor.clone();
        let msg_tx = self.msg_tx.clone();
        self.build = Some(spawn(async move {
            if let Err(err) = task_executor.start_lambda_task(&lambda_task).await.unwrap() {
                err_exit(format!("lambda task failure: {err}").as_str())
            }
            if let Some(completed) = lambda_task.completed {
                let _ = completed.send(());
            }
            msg_tx.send(TaskQueueMessage::Dequeue).unwrap();
        }));
    }

    pub fn try_next(&mut self) {
        self.build = None;
        if let Some(lambda_task) = self.queue.pop_front() {
            self.now(lambda_task);
        }
    }
}

pub struct BuildQueue {
    msg_tx: UnboundedSender<TaskQueueMessage>,
}

impl BuildQueue {
    pub fn new(executor: Arc<TaskExecutor>) -> Self {
        let (msg_tx, msg_rx) = unbounded_channel();
        let mut event_loop = BuildQueueEventLoop::new(executor, msg_rx, msg_tx.clone());
        spawn(async move { event_loop.start().await });
        Self { msg_tx }
    }

    pub fn enqueue(&mut self, lambda_task: LambdaTask) {
        self.msg_tx
            .send(TaskQueueMessage::Enqueue(lambda_task))
            .unwrap();
    }
}
