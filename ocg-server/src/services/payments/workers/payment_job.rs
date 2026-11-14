//! Background processing for provider-mediated payment jobs.

use anyhow::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, instrument, warn};

use crate::{
    config::HttpServerConfig,
    db::{DynDB, payments::ClaimedPaymentJob},
    services::{
        notifications::DynNotificationsManager,
        workers::claim_loop::{self, ClaimLoopConfig},
    },
    types::payments::PaymentJobKind,
};

use super::super::{DynPaymentsProvider, notification_composer::PaymentsNotificationComposer};

mod refund;

#[cfg(test)]
mod tests;

/// Number of workers that process payment jobs.
const NUM_WORKERS: usize = 2;

/// Starts provider-mediated payment job workers.
pub(in crate::services::payments) fn start(
    db: &DynDB,
    notifications_manager: DynNotificationsManager,
    payments_provider: Option<&DynPaymentsProvider>,
    server_cfg: HttpServerConfig,
    task_tracker: &TaskTracker,
    cancellation_token: &CancellationToken,
) {
    let notification_composer =
        PaymentsNotificationComposer::new(db.clone(), notifications_manager, server_cfg);

    // Start provider workers even when this deployment has no configured provider
    for _ in 0..NUM_WORKERS {
        let worker = Worker {
            cancellation_token: cancellation_token.clone(),
            db: db.clone(),
            notification_composer: notification_composer.clone(),
            payments_provider: payments_provider.cloned(),
        };
        task_tracker.spawn(async move {
            worker.run().await;
        });
    }
}

/// Processes durable payment jobs for the configured provider.
struct Worker {
    /// Coordinates graceful worker shutdown.
    cancellation_token: CancellationToken,
    /// Persists durable payment job lifecycle transitions.
    db: DynDB,
    /// Enqueues attendee notifications after local refund finalization.
    notification_composer: PaymentsNotificationComposer,

    /// Provider used to process payment jobs when configured.
    payments_provider: Option<DynPaymentsProvider>,
}

impl Worker {
    /// Processes payment jobs until graceful shutdown.
    async fn run(&self) {
        // Delegate payment-specific claim cadence while preserving this error boundary
        claim_loop::run(
            &self.cancellation_token,
            ClaimLoopConfig::default(),
            || self.process_next_payment_job(),
            |err| {
                error!(error = %err, "error processing payment job");
                None
            },
        )
        .await;
    }

    /// Claims and processes one durable payment job.
    #[instrument(skip(self), err)]
    async fn process_next_payment_job(&self) -> Result<bool> {
        // Leave durable jobs unclaimed when this worker has no configured provider
        let Some(provider_adapter) = self.payments_provider.as_ref() else {
            return Ok(false);
        };
        let configured_provider = provider_adapter.provider();

        let Some(job) = self
            .db
            .claim_payment_job(PaymentJobKind::EventPurchaseRefund, configured_provider)
            .await?
        else {
            return Ok(false);
        };

        debug!(
            attempt_count = job.attempt_count,
            event_purchase_id = %job.event_purchase_id,
            payment_job_id = %job.payment_job_id,
            payment_provider = %job.payment_provider,
            "processing payment job"
        );

        let crate::db::payments::ClaimedPaymentJobWork::EventPurchaseRefund { refund } = &job.work;
        if let Err(err) = refund::process(
            &self.db,
            &self.notification_composer,
            provider_adapter,
            &job,
            refund,
        )
        .await
        {
            self.release_claim(&job, &err).await;
            return Err(err);
        }

        Ok(true)
    }

    /// Releases the current claim without hiding the processing error.
    async fn release_claim(&self, job: &ClaimedPaymentJob, err: &anyhow::Error) {
        if let Err(record_err) = self
            .db
            .record_payment_job_failure(job.payment_job_id, job.claim_id, err.to_string())
            .await
        {
            warn!(error = %record_err, "failed to release payment job claim");
        }
    }
}
