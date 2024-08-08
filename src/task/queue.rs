// use std::sync::Arc;
//
// use crate::task::lambda::LambdaTask;
// use tokio::sync::mpsc::{Receiver, Sender};
// use tokio::sync::{mpsc, Notify};
// use tokio::task::JoinHandle;
//
// enum TaskQueueMessage {
//     AddTask(LambdaTask),
// }
//
// struct TaskQueueEventLoop {
//     msg_rx: Receiver<TaskQueueMessage>,
//     notify: Arc<Notify>,
//     running: JoinHandle<()>,
//     tasks: Vec<LambdaTask>,
// }
//
// pub struct TaskQueue {
//     msg_tx: Sender<TaskQueueMessage>,
// }
//
// impl TaskQueueEventLoop {
//     fn new() -> (Self, Sender<TaskQueueMessage>) {
//         let (msg_tx, msg_rx) = mpsc::channel::<TaskQueueMessage>(10);
//         let notify = Arc::new(Notify::new());
//         let running = Self::create_sleeping_task(notify.clone());
//         let event_loop = Self {
//             msg_rx,
//             notify,
//             running,
//             tasks: Vec::new(),
//         };
//         (event_loop, msg_tx)
//     }
//
//     async fn start(&mut self) {
//         loop {
//             tokio::select! {
//                 task_msg_opt = self.msg_rx.recv() => {
//                     self.handle_queue_message(task_msg_opt.unwrap());
//                 }
//                 _ = &mut self.running => {
//                     println!("finished task!");
//                 }
//             }
//         }
//     }
//
//     fn handle_queue_message(&mut self, msg: TaskQueueMessage) {
//         match msg {
//             TaskQueueMessage::AddTask(source_update) => self.queue_task(source_update),
//         }
//     }
//
//     fn queue_task(&mut self, source_update: LambdaTask) {
//         println!("queued task");
//         if self.tasks.is_empty() {
//             let notify = self.notify.clone();
//             self.running = Self::create_sleeping_task(notify.clone());
//         } else {
//             self.tasks.push(source_update);
//         }
//     }
//
//     fn create_sleeping_task(notify: Arc<Notify>) -> JoinHandle<()> {
//         tokio::spawn(async move { notify.notified().await })
//     }
// }
//
// impl TaskQueue {
//     pub fn new() -> Self {
//         let (mut event_loop, msg_tx) = TaskQueueEventLoop::new();
//         tokio::spawn(async move { event_loop.start().await });
//         Self { msg_tx }
//     }
//
//     pub fn add_task(&mut self, task: LambdaTask) -> Result<(), anyhow::Error> {
//         self.msg_tx.try_send(TaskQueueMessage::AddTask(task))?;
//         Ok(())
//     }
// }
