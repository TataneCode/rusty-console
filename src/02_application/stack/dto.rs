use crate::application::error::AppError;
use crate::domain::stack::StackContainerState;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct StackContainerDto {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: StackContainerState,
    pub status: String,
    pub ports: String,
    pub can_start: bool,
    pub can_stop: bool,
}

impl StackContainerDto {
    pub fn state_display(&self) -> &'static str {
        match self.state {
            StackContainerState::Running => "Running",
            StackContainerState::Paused => "Paused",
            StackContainerState::Stopped => "Stopped",
            StackContainerState::Exited => "Exited",
            StackContainerState::Dead => "Dead",
            StackContainerState::Created => "Created",
            StackContainerState::Removing => "Removing",
            StackContainerState::Restarting => "Restarting",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImagePullProgress {
    pub image: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

impl ImagePullProgress {
    pub fn percent(&self) -> u64 {
        if self.total_bytes == 0 {
            return 0;
        }
        (self.downloaded_bytes.min(self.total_bytes) * 100) / self.total_bytes
    }
}

#[derive(Debug)]
pub enum ImagePullEvent {
    Progress(ImagePullProgress),
    Completed(Result<(), AppError>),
}

#[derive(Debug)]
pub struct ImagePullSubscription {
    receiver: mpsc::Receiver<ImagePullEvent>,
    tasks: Vec<JoinHandle<()>>,
    pending_completed: Option<Result<(), AppError>>,
}

impl ImagePullSubscription {
    pub fn new(receiver: mpsc::Receiver<ImagePullEvent>, tasks: Vec<JoinHandle<()>>) -> Self {
        ImagePullSubscription {
            receiver,
            tasks,
            pending_completed: None,
        }
    }

    #[cfg(test)]
    pub fn completed(result: Result<(), AppError>) -> Self {
        ImagePullSubscription {
            receiver: {
                let (_sender, receiver) = mpsc::channel(1);
                receiver
            },
            tasks: Vec::new(),
            pending_completed: Some(result),
        }
    }

    pub fn try_recv(&mut self) -> Result<ImagePullEvent, mpsc::error::TryRecvError> {
        if let Some(result) = self.pending_completed.take() {
            return Ok(ImagePullEvent::Completed(result));
        }
        self.receiver.try_recv()
    }

    pub fn abort(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

impl Drop for ImagePullSubscription {
    fn drop(&mut self) {
        self.abort();
    }
}

#[derive(Debug, Clone)]
pub struct StackDto {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub containers: Vec<StackContainerDto>,
}
