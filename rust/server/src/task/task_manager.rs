use crate::error::{ErrorCode, Result};
use crate::task::env::GlobalTaskEnv;
use crate::task::task::{TaskExecution, TaskId};
use rayon::{ThreadPool, ThreadPoolBuilder};
use risingwave_proto::plan::PlanFragment;
use risingwave_proto::task_service::{TaskId as ProtoTaskId, TaskSinkId as ProtoSinkId};
use std::collections::HashMap;
use std::sync::Mutex;

pub(crate) struct TaskManager {
    worker_pool: ThreadPool,
    tasks: Mutex<HashMap<TaskId, Box<TaskExecution>>>,
    env: GlobalTaskEnv,
}

impl TaskManager {
    pub(crate) fn new(env: GlobalTaskEnv) -> Self {
        let worker_pool = ThreadPoolBuilder::default().num_threads(2).build().unwrap();
        TaskManager {
            worker_pool,
            tasks: Mutex::new(HashMap::new()),
            env,
        }
    }

    pub(crate) fn fire_task(&mut self, tid: &ProtoTaskId, plan: PlanFragment) -> Result<()> {
        let tsk = TaskExecution::new(tid, plan, self.env.clone());
        self.tasks
            .lock()
            .unwrap()
            .entry(tsk.get_task_id().clone())
            .or_insert_with(|| Box::new(tsk))
            .async_execute(&mut self.worker_pool)?;
        Ok(())
    }

    // NOTE: For now, draining out data from the task will block the grpc thread.
    // By design, `take_data` is an future, but since it's unsafe to mix other
    // future runtimes (tokio, eg.) with grpc-rs reactor, we
    pub(crate) fn take_task(&mut self, sid: &ProtoSinkId) -> Result<Box<TaskExecution>> {
        let task_id = TaskId::from(sid.get_task_id());
        let tsk = match self.tasks.lock().unwrap().remove(&task_id) {
            Some(t) => t,
            None => {
                return Err(
                    ErrorCode::InternalError(format!("No such task {:?}", &task_id)).into(),
                );
            }
        };
        Ok(tsk)
    }

    #[cfg(test)]
    pub(crate) fn get_global_env(&self) -> &GlobalTaskEnv {
        &self.env
    }
}

#[cfg(test)]
mod tests {
    use crate::task::test_utils::{ResultChecker, TestRunner};

    #[test]
    fn test_select_all() {
        let mut runner = TestRunner::new();
        runner
            .prepare_table()
            .create_table_int32s(3)
            .insert_i32s(&[1, 4, 2])
            .insert_i32s(&[2, 3, 3])
            .insert_i32s(&[3, 4, 4])
            .insert_i32s(&[4, 3, 5])
            .run();

        let res = runner.prepare_scan().scan_all().run();
        ResultChecker::new()
            .add_i32_column(false, &[1, 2, 3, 4])
            .add_i32_column(false, &[4, 3, 4, 3])
            .add_i32_column(false, &[2, 3, 4, 5])
            .check_result(res);
    }
}
