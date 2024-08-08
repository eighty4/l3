pub enum TaskContinuation<T> {
    Finished,
    QueueTask(T),
    QueueTasks(Vec<T>),
}

pub trait TaskWorkflow<T> {
    fn on_completed_task(task: T) -> TaskContinuation<T>;
    fn on_failed_task(task: T) -> TaskContinuation<T>;
}
