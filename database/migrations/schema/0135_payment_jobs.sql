-- Reserved migration number.
--
-- The upstream payment-job schema assumes direct charges and is incompatible
-- with GOUP's Stripe Connect destination-charge model. The Connect-compatible
-- durable refund queue is introduced by 0136_connect_payment_jobs.sql.
select 1;
