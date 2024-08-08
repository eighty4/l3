use std::time::Instant;

pub struct TaskEvent<T> {
    kind: T,
    timestamp: Instant,
    status: TaskStatus,
}

pub enum TaskStatus {
    Completed,
    Failed,
    Queued,
    Running,
}
