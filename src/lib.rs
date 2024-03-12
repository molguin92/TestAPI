use jwt_simple::claims::Claims;
use jwt_simple::prelude::{HS256Key, MACLike, NoCustomClaims};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;

use rand::random;
use rand_derive2::RandGen;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid;

const TOKEN_DURATION: Duration = Duration::from_secs(300);

pub enum ApiError {
    AuthError,
    APIError(String),
    NoSuchTask,
    IncorrectResult(i8),
}

#[derive(RandGen, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TaskOp {
    Max,
    Min,
}

impl TaskOp {
    fn apply(&self, args: &Vec<i8>) -> i8 {
        match self {
            TaskOp::Max => args.iter().max().unwrap().clone(),
            TaskOp::Min => args.iter().min().unwrap().clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APITask {
    task_id: String,
    token: String,
    op: TaskOp,
    args: Vec<i8>,
}

impl APITask {
    fn new(key: &HS256Key) -> Result<Self, ApiError> {
        let claims = Claims::create(TOKEN_DURATION.into());

        let mut num_args: u8;
        loop {
            num_args = random();
            if num_args > 0 {
                break;
            }
        }

        let mut args = Vec::with_capacity(num_args as usize);
        for _ in 0..num_args {
            args.push(random());
        }

        Ok(Self {
            token: key
                .authenticate(claims)
                .map_err(|e| ApiError::APIError(e.to_string()))?,
            task_id: uuid::Builder::from_random_bytes(random())
                .into_uuid()
                .to_string(),
            op: random(),
            args,
        })
    }
}

struct Task {
    task: APITask,
    expected: i8,
}

impl Task {
    fn new(key: &HS256Key) -> Result<Self, ApiError> {
        let api_task = APITask::new(key)?;
        let expected = api_task.op.apply(&api_task.args);

        Ok(Self {
            task: api_task,
            expected,
        })
    }
}

pub struct API {
    key: HS256Key,
    tasks: RwLock<HashMap<String, Task>>,
}

impl API {
    pub fn new() -> Arc<API> {
        Arc::new(Self {
            key: HS256Key::generate(),
            tasks: Default::default(),
        })
    }

    pub async fn new_task(self: &Arc<API>) -> Result<APITask, ApiError> {
        let task = Task::new(&self.key)?;
        let api_task = (&task.task).clone();
        let task_id = (&task.task.task_id).clone();

        self.tasks.write().await.insert(task_id.clone(), task);
        Ok(api_task)
    }

    pub async fn validate_result<S: AsRef<str>>(
        self: &Arc<API>,
        task_id: S,
        token: S,
        result: i8,
    ) -> Result<(), ApiError> {
        let _ = self.key
            .verify_token::<NoCustomClaims>(token.as_ref(), None)
            .map_err(|e| ApiError::AuthError)?;

        match self.tasks.read().await.get(task_id.as_ref()) {
            None => Err(ApiError::NoSuchTask),
            Some(task) => {
                if task.expected == result {
                    Ok(())
                } else {
                    Err(ApiError::IncorrectResult(task.expected))
                }
            }
        }
    }
}
