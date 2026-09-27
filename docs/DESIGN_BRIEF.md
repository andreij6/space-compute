# Design Brief — Space Compute (working name) — MVP

This brief describes **what** we're building for a first, minimal version. How it looks is up to you: style, layout, branding and tone are yours to invent. We're keeping the scope small on purpose, so please design only what's listed here.

## The product in one paragraph

This is a citizen-science website for astronomy, similar to Galaxy Zoo, with a twist. Each user spawns a personal **AI "Agent Amateur Astronomer" (AAA)**. The AAA looks at real telescope images of galaxies, classifies them, flags possible discoveries, and reviews other AAAs' discoveries. **All of that work is done by the user's own AI agent** running on their computer. The agent pulls images and the criteria for judging them directly. The website is where people **see the results** of their agent's decisions. There are no screens for classifying or reviewing by hand. Each AAA is a small program on a blockchain (Internet Computer) that the user owns and keeps "fueled" with **cycles**. If the fuel runs out, the AAA pauses.

## Who uses it

1. **Owners**: people with one AAA. They set up their AAA, connect their agent, keep it fueled, and check in to see what it decided.
2. **Visitors**: people who aren't signed in and are browsing public discoveries.

## Things to design for

- **AAA**: name, simple avatar, status (active / low fuel / paused), fuel level, reputation score, **tier (level)**, **XP progress to the next tier**, **badges**, and a few stats (classified, discoveries, reviews, discoveries credited on).
- **Image**: a **James Webb Space Telescope** color image of one object, usually a distant galaxy near the center. It comes with a data panel: sky coordinates, the JWST field and program, which infrared filters make up the colors, estimated redshift / distance ("light left it 11 billion years ago"), mass, and download links for the scientific files.
- **Classification**: answers to a short question tree of about 3–5 questions (e.g., "Smooth or featured?", "Spiral arms?", "Anything odd?").
- **Discovery**: a flagged image with a category (merger, lens candidate, unusual shape, artifact, other) and a written rationale. Status: *Under review → Confirmed / Rejected.*
- **Review**: a vote (agree / disagree) with a short rationale.
- **Citation**: the permanent credit on a discovery. This is a core feature, not decoration. It lists the discovering AAA and every reviewing AAA (with each one's owner and vote), a permanent discovery ID (e.g., `SC-2026-000123`), the outcome and date, and a copyable "cite this" line. Think of it as the discovery's author list. It's the reason people run an AAA.
- **Tier**: about 5 levels an AAA climbs by doing accurate work. Names and insignia are open for you to propose.
- **Top-up**: a payment that becomes fuel for the AAA. It can be **ICP** (any amount), or a **$5 fuel pack** paid by **card, BTC or ETH**. It has an amount, the fuel delivered, the method, whether it was manual or automatic, and a status.
- **Auto top-up (subscription)**: either an ICP pre-approval with a monthly limit (the AAA refuels itself when low), or a **$5/month card subscription**. Both can be cancelled at any time.
- **Badge**: an achievement. Starter set: *First Light* (first classification), *First Find* (first discovery), *Confirmed Discoverer*, *Peer Reviewer* (10 reviews), *Sharp Eye* (streak of correct answers on known test images). More will be added later, so the badge style needs to scale.

## Screens (10 public/owner + admin console)

### Public
1. **Landing page**: the pitch, a short "how it works" section, a few recent confirmed discoveries, and a clear "Spawn your AAA" call to action.
2. **Discovery feed**: a grid or list of discoveries, filterable by category and status.
3. **Discovery detail**: the image (zoomable), category, rationale, status, reviews, and a prominent **citation block** that credits the discoverer and all reviewers (with tier insignia), plus a copyable citation line and permalink.
4. **AAA public profile**: avatar, name, tier and XP progress, reputation, badges (earned plus locked or greyed-out ones), stats, and **credits**: every discovery this AAA is cited on, as discoverer or reviewer.
5. **Leaderboard**: a single all-time ranking of AAAs by XP, showing tier, confirmed discoveries and reviews. Keep it simple.

### Owner (signed in with Internet Identity; sign-in is just a button, since the login screen is provided externally)
6. **Spawn AAA flow**: name it, fund it with ICP, BTC or ETH, **or enter an invite code** (free starter fuel). The card option is designed but **hidden at launch**.
7. **Dashboard**: the owner's home. It should show at a glance:
   - AAA status and a fuel gauge, with a top-up action, a low-fuel warning and auto top-up status
   - whether the agent is connected, and when it was last active
   - recent activity (images classified, discoveries flagged, reviews given)
   - the owner's discoveries and their review status
   - the reviews the agent has given, and how they compared with the final outcome
   - tier and XP progress, recently earned badges, and new citations ("your AAA was credited on SC-…")
8. **Connect your agent**: step-by-step setup instructions with copyable commands, plus a connection status.
9. **Activity & records**: the full history of everything the agent did, filterable by type (classification / discovery / review). Selecting an item shows what the agent decided: the image, the criteria it was given, its answers, and its rationale. This is read-only.
10. **Fuel & billing**: where owners pay. It includes:
   - the current fuel level and an estimate of how long it will last
   - **one-time top-up**, with method tabs:
     - **ICP**: any amount; pay from a connected wallet or a shown address/QR code
     - **Card**: 1–4 $5 fuel packs via Stripe checkout
     - **BTC**: deposit address + QR, with confirmation progress
     - **ETH**: pay with a browser wallet, with a minting-progress indicator
     
     Each tab shows roughly how many days of fuel the payment buys.
   - **auto top-up**: either an ICP monthly limit plus a low-fuel threshold (approved in the wallet, showing how much of the limit is used) or a $5/month card subscription (manage or cancel via Stripe)
   - a notice when card/crypto fuel is temporarily unavailable (ICP still works)
   - top-up history (amount, fuel delivered, manual or auto, status)

   The payment step (amount → method → pay → confirmed) is a shared component, reused in the Spawn flow and in top-up actions. Design it so that more payment methods could be added later without a redesign.

### Public extras
- About / Terms / Privacy / Credits pages (text-heavy, simple), including a small "observatory runway" widget showing how long the app's own fuel lasts (from the owner-funded treasury).
- A consent banner for analytics.

### Admin (only visible to site admins; functional over pretty; reuse the style system)
11. **Admin console** with a left navigation:
   - **Overview:** health tiles (agents active, tasks/hour, discoveries under review, treasury balance vs floor, canister fuel), alerts, and big pause switches with a typed confirmation
   - **Agents:** a searchable table; a detail drawer with owner, status, operators, progress and recent activity; suspend/unsuspend
   - **Discoveries:** a queue including under-review and stalled items, corroborations, and reviewer accuracy on test discoveries
   - **Data:** JWST subject counts by field, test/gold status, question-tree versions
   - **Payments & treasury:** balances, daily caps, failed payments (with a resume button), card fraud blocks
   - **Releases:** agent software versions, with an approve flow
   - **Settings:** parameters, with a before/after diff, feature flags (e.g. card payments on/off), and the admin list
   - **Invites:** mint and track invite codes
   - **Treasury:** ICP balance and deposits, per-canister runway, the app's fuel runway, deposit-pause state
   - **Moderation:** rename offensive agent names
   - **Audit log:** who did what, when

## States to cover

- Empty: new AAA with no activity, no discoveries, no pending reviews.
- Fuel: healthy, low, paused (out of fuel; the data is safe but the AAA can't work).
- Agent: not connected / connected.
- Discovery: under review (with progress, e.g., "2 of 3 reviews"), confirmed, rejected.
- Citation: in progress (discovery under review; reviewers are still being added) vs. final (frozen after consensus).
- Progression: badge earned, tier up, and locked badges.
- Payment: awaiting payment, processing, waiting for BTC confirmations (up to ~1 h), ETH minting (~20 min), completed, failed, card declined, and card/crypto temporarily unavailable.
- Auto top-up: off, active, monthly limit reached, and allowance revoked or insufficient.
- Loading: blockchain actions can take a few seconds.

## Considerations

- **Images are the star.** JWST images are gorgeous but tiny, faint and full of neighbours. Make them look good, easy to zoom, and put the key numbers in plain language right beside them.
- **Owners watch; agents work.** Nothing on the site lets a person classify or review. The dashboard should work as a "what happened while I was away" summary, and the records view should make the agent's reasoning easy to follow.
- **Make the tech approachable.** Explain "cycles" as fuel without hiding the real term.
- **Credit is the reward.** Citations should feel permanent and prestigious, like an author list on a paper. Tiers and badges should feel earned, not childish, because this is real science.
- **Payments should feel simple.** Show the price in the currency being paid (ICP, $, BTC, ETH) alongside what it buys, e.g. "≈ 30 days of fuel". Owners shouldn't need to understand cycles to top up.
- **Privacy-friendly analytics:** a simple consent banner ("Allow analytics" / "Essential only").
- **Desktop first.** Public pages and the dashboard should still be readable on mobile.

## Deliverables

- Mocks for the 10 screens, prioritizing the **Dashboard, Fuel & billing, Discovery detail (with citation), AAA profile, Activity & records and Landing** screens. The Leaderboard is lowest priority.
- The citation block as a reusable component.
- The payment step as a reusable component, covering every payment state.
- Tier insignia for about 5 levels, and designs for the 5 starter badges, in a style that can grow to many more.
- The key states above, especially fuel and discovery status.
- A minimal style guide: colors, type, and core components.

## Not in the MVP (please don't design these yet)

Any screen for classifying or reviewing by hand, monthly/seasonal/category leaderboards, a badge catalog beyond the starter 5, notifications, multiple AAAs per owner, drawing or annotating regions on images, discussion or comments, settings pages, and full account management.

## Reference

- Galaxy Zoo (the inspiration): https://www.zooniverse.org/projects/zookeeper/galaxy-zoo
