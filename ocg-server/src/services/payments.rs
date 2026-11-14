//! Payments service management and provider integrations.
//!
//! OCG uses a provider-agnostic payments domain model while running with a
//! single configured payments provider at a time. Provider-specific behavior
//! should stay isolated inside provider modules, while shared checkout,
//! refund, and webhook flows stay generic.

mod manager;
mod notification_composer;
mod provider;
mod refund_recorder;
mod webhook_reconciler;
mod workers;

pub(crate) use manager::{
    ApproveRefundRequestInput, CompleteRefundRecoveryInput, DynPaymentsManager, HandleWebhookError,
    PgPaymentsManager, RejectRefundRequestInput, RequestRefundInput,
};
pub(crate) use provider::{
    CheckoutSession, CreateCheckoutSessionInput, DynPaymentsProvider, FindRefundInput,
    PaymentsWebhookEvent, RefundPaymentInput, RefundPaymentResult, RefundPaymentStatus,
    build_payments_provider,
};
/// Starts all durable payment workers.
pub(crate) fn start_payment_workers(
    db: &crate::db::DynDB,
    notifications_manager: crate::services::notifications::DynNotificationsManager,
    payments_provider: Option<&DynPaymentsProvider>,
    server_cfg: crate::config::HttpServerConfig,
    task_tracker: &tokio_util::task::TaskTracker,
    cancellation_token: &tokio_util::sync::CancellationToken,
) {
    workers::payment_job::start(
        db,
        notifications_manager,
        payments_provider,
        server_cfg,
        task_tracker,
        cancellation_token,
    );
    workers::recovery::start(db, task_tracker, cancellation_token);
}

#[cfg(test)]
pub(crate) use manager::MockPaymentsManager;
#[cfg(test)]
pub(crate) use provider::{MockPaymentsProvider, PaymentsProvider};
