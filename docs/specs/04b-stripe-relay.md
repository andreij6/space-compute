# 04b — Stripe relay (off-chain, Cloudflare Worker, TypeScript)

> **Status: DEFERRED (owner, 2026-09-27).** Card payments are hidden at launch (`features.card = false`, ADR-20). This spec is kept for later. In the MVP only the canister-side `stripe_credit` endpoint is built, tested with a mock relay principal, and disabled by the flag. The relay Worker, Stripe account and card UI are **not** built. See the deferred tasks D1–D4 in 10.

Card payments need Stripe secrets, and canister state is readable by node providers (see the `canister-security` skill, Pitfall 9). So the only off-chain component in the system is a **tiny stateless relay** that holds the Stripe keys and forwards verified payment events to `payments.stripe_credit`.

## 1. Endpoints
| Endpoint | Caller | Does |
|---|---|---|
| `POST /checkout` | frontend (the user's browser) | Body `{ aaa, packs (1-4), mode: "payment" \| "subscription" }`. Checks the AAA via `platform.get_aaa_public(aaa)` (query), then creates a Stripe Checkout Session (price = `STRIPE_PRICE_FUEL_PACK` $5 × qty, or the subscription price) with `metadata.aaa` and `client_reference_id = aaa`. Returns `{ url }`. Rate limit: 10 per IP per hour. |
| `POST /webhook` | Stripe | Verifies the `Stripe-Signature` with `STRIPE_WEBHOOK_SECRET` (rejects failures and events older than 5 min). It handles `checkout.session.completed` (mode=payment, `payment_status=paid`) and `invoice.paid` (subscription renewals, first invoice included). It then calls `payments.stripe_credit{ stripe_ref: session.id \| invoice.id, aaa, packs, usd_cents, kind }` using the **relay identity** (Ed25519 key in Worker secrets, registered as `config.stripe_relay`). It returns 2xx only after the canister returns `Ok` or `Duplicate`, so Stripe retries otherwise. |
| `POST /portal` | frontend | Creates a Stripe billing-portal session for managing or cancelling the subscription. The Stripe customer is looked up by `metadata.aaa`. |

## 2. Secrets & config
`STRIPE_SECRET_KEY` (restricted key: Checkout, Billing, Customers), `STRIPE_WEBHOOK_SECRET`, `STRIPE_PRICE_FUEL_PACK`, `STRIPE_PRICE_FUEL_SUB`, `RELAY_IDENTITY_PEM`, `PAYMENTS_CANISTER_ID`, `PLATFORM_CANISTER_ID`, `IC_HOST`. The relay stores **no database**: Stripe is the source of truth, and the canister is idempotent on `stripe_ref`.

## 3. Threats specific to the relay
| Threat | Mitigation |
|---|---|
| Relay key stolen → forged credits drain the treasury | Canister caps: `treasury_daily_cap_e8s`, `per_aaa_daily_packs`, and a `stripe_daily_usd_cap`. `usd_cents` must equal `packs × 500`. **Daily reconciliation job**: Stripe's paid sessions/invoices vs `stripe_credit` ops; a mismatch → alert + `admin_pause_non_icp`. Key rotation: `admin_set_stripe_relay(principal)`. |
| Chargebacks after cycles are delivered | Cycles can't be clawed back; this is accepted at $5 granularity. Stripe Radar is enabled. More than 2 disputes per owner → the owner's AAA is blocked from card packs (`admin_block_card(aaa)`). |
| Webhook replay | Signature timestamp tolerance plus canister idempotency. |

## 4. Acceptance
1. Stripe CLI `stripe trigger checkout.session.completed` (test mode, staging) → one op → the AAA receives cycles. Re-sending the same event → no second op.
2. A tampered signature → 400, and no canister call.
3. A subscription renewal (`invoice.paid`, test clock) → one pack per month.
4. The reconciliation script reports 0 mismatches on staging after 20 test payments.
