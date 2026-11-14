-- Tests cover configured and unconfigured external-payment states explicitly.
-- Remove the production default so each transactional test can seed its state.
delete from external_payments_config;
