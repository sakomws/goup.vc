//! Background recovery for payment claims abandoned by interrupted workers.

use std::time::Duration;

use anyhow::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, warn};

use crate::{
    db::DynDB,
    services::workers::{WorkerIteration, run_worker},
};

#[cfg(test)]
mod tests;

/// Number of workers that recover stale payment claims.
const NUM_WORKERS: usize = 1;

/// Interval between stale-claim recovery attempts.
const PAUSE_ON_RECOVERY: Duration = Duration::from_mins(1);

/// Starts payment claim recovery independently from provider configuration.
pub(in crate::services::payments) fn start(
    db: &DynDB,
    task_tracker: &TaskTracker,
    cancellation_token: &CancellationToken,
) {
    for _ in 0..NUM_WORKERS {
        let worker = Worker {
            cancellation_token: cancellation_token.clone(),
            db: db.clone(),
        };
        task_tracker.spawn(async move {
            worker.run().await;
        });
    }
}

/// Recovers stale claims across the durable payment job queue.
struct Worker {
    /// Coordinates graceful recovery-worker shutdown.
    cancellation_token: CancellationToken,
    /// Requeues durable claims abandoned by interrupted workers.
    db: DynDB,
}

impl Worker {
    /// Requeues stale payment claims until graceful shutdown.
    async fn run(&self) {
        run_worker(&self.cancellation_token, || async {
            // Sweep every payment queue before the fixed recovery cadence
            self.sweep_stale_claims().await;
            WorkerIteration::Pause(PAUSE_ON_RECOVERY)
        })
        .await;
    }

    /// Attempts payment job recovery.
    async fn sweep_stale_claims(&self) {
        // Recover abandoned payment job claims in one database sweep
        Self::report_recovery(self.db.requeue_stale_payment_job_claims().await);
    }

    /// Reports recovery activity and sweep failures at the worker boundary.
    fn report_recovery(result: Result<i32>) {
        match result {
            Ok(recovered) if recovered > 0 => {
                warn!(recovered, "requeued stale payment job claims");
            }
            Ok(_) => {}
            Err(err) => error!(error = %err, "error recovering payment job claims"),
        }
    }
}
