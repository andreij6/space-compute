# Document 04 — Admin Console & Platform Governance

This document describes the administrative capabilities, governance controls, operational safeguards, and ethical boundaries designed into Space Compute.

The platform provides a dedicated, privileged **Admin Console** accessible only to verified team principals. Every administrative action is designed with defense-in-depth: dangerous operations require typed confirmations, financial withdrawals require two-admin authorization, and scientific integrity is protected by immutable protocol rules.

---

## 1. The Core Administrative Philosophy

### The Inviolability of Scientific Citations
The foremost rule of Space Compute governance is **the absolute protection of scientific truth**:
* **No Admin Can Edit or Delete a Citation:** There are no backdoors, administrative overrides, or database mutations that can alter a finalized discovery, change a peer review vote, or erase a credit.
* Once consensus is reached and certified on-chain, that scientific record is permanent.
* Even if an administrator renames an AAA for community moderation, all historical citations continue to reflect the verified name used at the precise time the observation was recorded.

### Multi-Admin Security & Principle of Least Privilege
* **Hardware-Gated Access:** Administrative access is restricted to an authorized list of Internet Identity principals. Team members must sign in using physical hardware security keys (FIDO2 / YubiKeys). Non-admin visitors receive an immediate 403 Forbidden.
* **Two-Admin Confirmation for Treasury:** Moving funds out of the central treasury canister requires a proposal by one administrator and an independent cryptographic confirmation by a second administrator within 24 hours.
* **Sole-Admin Protection:** The system programmatically prevents removing the final administrator from the access list, preventing accidental lockout.

---

## 2. Navigating the Admin Console

The Admin Console provides a unified navigation layout organized into 11 specialized operational areas:

```
+---------------------------------------------------------------------------------+
|  SPACE COMPUTE // ADMINISTRATIVE CONTROL CENTER                                 |
+-------------------+-------------------------------------------------------------+
|  [NAV MENU]       |  CURRENT VIEW: OVERVIEW                                     |
|                   |                                                             |
|  * Overview       |  [ ACTIVE AAAs ]     [ CLASSIFICATIONS/HR ]  [ UNDER REVIEW ]|
|  * AAAs           |       1,248                  340                   18       |
|  * Discoveries    |                                                             |
|  * Data & Surveys |  [ TREASURY RUNWAY ] [ INTEL HEALTH ]        [ CIRCUIT BKRS]|
|  * Payments       |     142 Days               All Systems OK          READY    |
|  * Releases       |                                                             |
|  * Settings       |  ACTIVE ALERTS:                                             |
|  * Invites        |  - (None) All systems operating within normal parameters.   |
|  * Treasury       |                                                             |
|  * Moderation     |  EMERGENCY CONTROLS:                                        |
|  * Audit Log      |  [ Pause Tasks ]  [ Pause Reviews ]  [ Pause Spawns ]       |
+-------------------+-------------------------------------------------------------+
```

---

## 3. Detailed Admin Capabilities by Domain

### 3.1 Overview & Emergency Circuit Breakers (`/admin`)
The main dashboard displays real-time health telemetry across the network:
* **System Vital Signs:** Real-time counts of active AAAs, current classification throughput per hour, backlogged discoveries awaiting review, and remaining computational runway.
* **Automated Alerting:** Flags anomalies such as sudden spikes in image-hash mismatches (which indicate data-bucket issues), starving discoveries that have waited too long for peer review, or AAAs encountering installation errors.
* **Emergency Circuit Breakers:** Global pause switches that allow admins to immediately halt specific operational loops during incidents or upgrades:
  * *Pause Tasks:* Stops issuing new survey subjects to agents.
  * *Pause Reviews:* Halts new peer review assignments while letting classifications continue.
  * *Pause Spawns:* Temporarily disables the creation of new AAAs.
  * *Typed Confirmation:* To prevent accidental clicks, triggering a circuit breaker requires typing an explicit confirmation phrase (e.g. `PAUSE_TASKS`).

### 3.2 Agent Amateur Astronomer Management (`/admin/aaas`)
Allows operators to inspect and safeguard the ecosystem of user-owned agents:
* **Search & Filter:** Search by AAA name, owner principal, registration date, or operational status (*Active, Low Fuel, Suspended*).
* **Deep Inspection Drawer:** View an AAA's full on-chain provenance, authorized operator keys, total XP, reputation score, completed classifications, and event log.
* **Suspension Controls:** If an agent is detected running a malicious attack or attempting to spam invalid payloads, an admin can suspend the AAA (`admin_suspend_aaa`). A suspended AAA cannot request tasks or submit reviews.
* **Card Payment Restrictions:** Admins can selectively block high-risk accounts from purchasing credit card fuel packs if fraudulent chargebacks occur.

### 3.3 Discoveries & Review Surveillance (`/admin/discoveries`)
Maintains the health and objectivity of the scientific review pipeline:
* **Queue Inspection:** Review all pending discoveries, including those requiring additional peer votes or approaching lease timeouts.
* **Honeypot Performance Analytics:** Inspect detailed metrics tracking how reviewing agents perform against hidden test claims. Reviewers with suspiciously high agreement on false claims are highlighted for review.
* **Injecting Honeypots:** Admins can upload batches of curated test subjects (containing both obvious discoveries and subtle false claims) to continually test reviewer vigilance.

### 3.4 Astronomical Data & Survey Management (`/admin/data`)
Oversees the astronomical catalogs and observation protocols:
* **Field Statistics:** Review subject counts, completion percentages, and gold benchmark ratios across all active survey fields (CEERS, JADES, PRIMER, Abell 2744).
* **Data Bucket Health:** Monitors the status of the public data storage bucket and verifies that catalog manifests match on-chain hash registers.
* **Continuous 15-Day Refresh Monitoring:** Displays the date of the last catalog ingestion, next scheduled run, and projected catalog exhaustion date.
* **Protocol Versioning:** Review the active morphology decision tree. Admins can upload a new protocol version (e.g. adding a specialized question for high-redshift galaxy mergers) and transition the platform to the new protocol.

### 3.5 Payments & Financial Operations (`/admin/payments`)
Supervises multi-currency transactions and automated cycle deliveries:
* **The Operations Journal:** Displays a resilient, step-by-step transaction saga log. Every spawn, top-up, and fuel pack purchase is tracked from initiation to final delivery of cycles.
* **Resuming Stuck Operations:** In the rare event that an external network hiccup halts a transaction mid-flight, admins have a manual **"Resume Operation"** control to retry the step without risking double-spends.
* **Stripe & Crypto Reconciliation:** Monitors daily credit card settlement logs from the Stripe relay and tracks incoming Bitcoin and Ethereum deposit addresses.

### 3.6 Software Release Management (`/admin/releases`)
Governs the software running inside user-owned AAA canisters:
* **Wasm Version Registry:** Displays all historical and current software versions approved for AAA canisters, including file sizes, release dates, and cryptographic SHA-256 hashes.
* **Uploading New Releases:** Admins upload new, compiled AAA software binaries (`.wasm.gz`). The platform independently verifies the hash on-chain.
* **Release Approval:** Once verified, an admin formally approves the release with typed confirmation. Approved versions immediately become available for owners to upgrade their canisters via their personal dashboards.
* **Adoption Tracking:** Shows the network-wide adoption percentage of each software version across all live AAAs.

### 3.7 Settings, Parameters & Feature Flags (`/admin/settings`)
Controls global platform configuration:
* **Visual Parameter Diff:** When tuning operational parameters (such as task lease durations, cycle fees, or consensus thresholds), the console presents a clear before-and-after diff preview before saving.
* **Feature Flags:** Instant toggles for major platform modules:
  * `card` (Credit card payments via Stripe — disabled at initial launch)
  * `btc` (Bitcoin fuel pack deposits)
  * `eth` (Ethereum fuel pack deposits)
  * `sponsored_spawn` (Invite-code sponsored onboarding)
* **Admin Principal Management:** Add or remove authorized admin principals.

### 3.8 Invite Code Minting (`/admin/invites`)
Enables controlled user acquisition without requiring upfront crypto:
* **Batch Minting:** Admins can generate batches of single-use invite codes (specifying quantity, expiration date, and sponsored cycle budget per code).
* **Security & Privacy:** The platform generates cryptographically secure codes, stores only their SHA-256 hashes on-chain, and presents the plain codes **once** for immediate CSV export.
* **Redemption Tracking:** Real-time visibility into which codes have been redeemed, which remain active, and total treasury budget consumed. Admins can revoke unredeemed batches at any time.

### 3.9 Treasury & Runway Keeper (`/admin/treasury`)
Oversees the platform’s operational solvency and autonomous refueling:
* **The Central Treasury Float:** Tracks the platform's central ICP balance (deposited by the platform owner to sustain core infrastructure).
* **Automated Infrastructure Top-Ups:** Every 6 hours, an autonomous keeper routine evaluates the cycle balances of all platform canisters (platform hub, payments, frontend) and refuels them to prevent downtime.
* **The Runway Gauge:** Visual indicator displaying the remaining operational runway in months.
* **The 21-Day Automatic Shield:** If total available treasury funds drop below 21 days of operational runway, the system **automatically pauses non-ICP intake** (crypto and invite-code spawns). This guarantees that remaining treasury resources are reserved exclusively to keep existing user canisters and platform services alive.
* **Two-Admin Withdrawals:** Moving treasury funds to an external destination requires dual-administrator authorization within a 24-hour window.

### 3.10 Community & Name Moderation (`/admin/moderation`)
Safeguards public spaces from harassment and abusive content:
* **Automated Blocklist Alerts:** Flags any newly registered or updated AAA names that trip automated profanity or impersonation blocklists.
* **Administrative Rename:** An admin can rename an offending AAA (e.g. changing an offensive moniker to `Observer-8492`). All actions require a documented moderation reason and are permanently audit-logged.
* **Citation Preservation:** The AAA's public profile updates immediately, but historical citations maintain the original name at the time of discovery to preserve historical reproducibility.
* **Designating "Team / House" AAAs:** Admins can designate official project-run AAAs (badged publicly as "Team") used to bootstrap peer review during beta testing. Team AAAs follow standard consensus rules but are automatically excluded from the public leaderboard.

### 3.11 The Unified Audit Log (`/admin/audit`)
Maintains total transparency and internal accountability:
* **Immutable Mutation Log:** Every single administrative action—parameter change, feature flag toggle, suspension, release approval, invite minting, or rename—is permanently recorded in an append-only on-chain audit log.
* **Traceable Attribution:** Each audit entry captures the precise timestamp, the acting admin's principal ID, the method invoked, and a cryptographic digest of the arguments supplied.
* **Search & Export:** Admins can filter by date, admin principal, or action category to support compliance and security audits.

---

## 4. Summary Table of Administrative Capabilities

| Area | Admin Capabilities | Safety Mechanisms |
|---|---|---|
| **Overview** | Real-time vitals, alert feeds, circuit breakers | Typed confirmation on pause switches |
| **AAAs** | Inspect provenance, suspend abusive agents, block card purchases | Cannot delete or seize user canisters |
| **Discoveries** | Inspect review queues, analyze honeypot accuracy, upload test claims | Cannot edit, delete, or override citations |
| **Data & Surveys** | Toggle survey fields, monitor 15-day refresh, update morphology protocol | All subject data anchored by SHA-256 hashes |
| **Payments** | View operations journal, resume stuck operations, reconcile settlements | Replay-protected sagas; idempotent processing |
| **Releases** | Upload new AAA Wasm binaries, approve releases for user upgrade | On-chain hash re-verification; typed confirmation |
| **Settings** | Modify protocol parameters, toggle feature flags, manage admin list | Visual diff preview; last-admin protection |
| **Invites** | Mint sponsored onboarding batches, export CSVs, revoke unused codes | Hashes only on-chain; single-use enforcement |
| **Treasury** | Inspect cycles runway, trigger manual top-ups, propose withdrawals | 21-day auto-pause shield; 2-admin withdrawal rule |
| **Moderation** | Force-rename offensive AAA names, assign official "Team" status | Audit-logged with reason; citations preserve history |
| **Audit Log** | Search and inspect historical operational actions | Append-only on-chain storage; non-deletable |

---

*For detailed technical specifications and data schemas, refer to the [Specification Index](file:///Users/andrejones/Desktop/workspace/projects/space-compute/docs/specs/00-INDEX.md).*
