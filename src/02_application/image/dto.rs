use crate::application::error::AppError;
#[derive(Debug, Clone)]
pub struct ImageDto {
    pub id: String,
    pub short_id: String,
    pub repository: String,
    pub tag: String,
    pub full_name: String,
    pub size: String,
    pub created: String,
    pub in_use: bool,
    pub is_dangling: bool,
    pub can_delete: bool,
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
    Completed(Result<ImagePullOutcome, AppError>),
}

#[derive(Debug)]
pub struct ImagePullOutcome {
    pub pulled: Vec<String>,
    pub up_to_date: Vec<String>,
}

impl ImagePullOutcome {
    pub fn empty() -> Self {
        ImagePullOutcome {
            pulled: Vec::new(),
            up_to_date: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ImagePullSubscription {
    receiver: tokio::sync::mpsc::Receiver<ImagePullEvent>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
    pending_completed: Option<Result<ImagePullOutcome, AppError>>,
}

impl ImagePullSubscription {
    pub fn new(
        receiver: tokio::sync::mpsc::Receiver<ImagePullEvent>,
        tasks: Vec<tokio::task::JoinHandle<()>>,
    ) -> Self {
        ImagePullSubscription {
            receiver,
            tasks,
            pending_completed: None,
        }
    }

    pub fn completed(outcome: Result<ImagePullOutcome, AppError>) -> Self {
        let (_sender, receiver) = tokio::sync::mpsc::channel(1);
        ImagePullSubscription {
            receiver,
            tasks: Vec::new(),
            pending_completed: Some(outcome),
        }
    }

    pub fn try_recv(&mut self) -> Result<ImagePullEvent, tokio::sync::mpsc::error::TryRecvError> {
        if let Some(outcome) = self.pending_completed.take() {
            return Ok(ImagePullEvent::Completed(outcome));
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
