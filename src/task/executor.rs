// use std::collections::HashMap;
//
// use tokio::sync::mpsc::{Receiver, Sender};
//
// use crate::code::source::update::SourceUpdate;
// use crate::lambda::RouteKey;
// use crate::notification::LambdaNotification;
// use crate::task::queue::TaskQueue;
//
// struct TaskExecutorEventLoop {
//     de_tx: Sender<LambdaNotification>,
//     src_rx: Receiver<SourceUpdate>,
//     tasks: HashMap<RouteKey, TaskQueue>,
// }
//
// pub struct TaskExecutor {}
//
// impl TaskExecutorEventLoop {
//     fn new(de_tx: Sender<LambdaNotification>, src_rx: Receiver<SourceUpdate>) -> Self {
//         Self {
//             de_tx,
//             src_rx,
//             tasks: HashMap::new(),
//         }
//     }
//
//     async fn start(&mut self) {
//         loop {
//             tokio::select! {
//                 source_update_opt = self.src_rx.recv() => {
//                     self.handle_source_update(source_update_opt.unwrap());
//                 }
//             }
//         }
//     }
//
//     fn handle_source_update(&mut self, source_update: SourceUpdate) {
//         // let route_key = &source_update.path.route_key();
//         // match self.tasks.get_mut(route_key) {
//         //     None => {
//         //         let mut q = TaskQueue::new();
//         //         let route_key = source_update.route_key.clone();
//         //         q.add_task(source_update).expect("queueing add task");
//         //         self.tasks.insert(route_key, q);
//         //     }
//         //     Some(q) => q.add_task(source_update).expect("queueing add task"),
//         // }
//     }
// }
//
// impl TaskExecutor {
//     pub fn new(de_tx: Sender<LambdaNotification>, src_rx: Receiver<SourceUpdate>) -> Self {
//         let mut event_loop = TaskExecutorEventLoop::new(de_tx, src_rx);
//         tokio::spawn(async move { event_loop.start().await });
//         Self {}
//     }
// }
