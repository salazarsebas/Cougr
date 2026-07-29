# On-Chain / Off-Chain Boundary Guide

> ⏳ **This page is being written.**
>
> Named as the **second-highest-priority** missing document in `docs/strategy/08-ux-strategy.md` and `docs/strategy/12-documentation-architecture.md`.
>
> **Tracked in:** [`salazarsebas/Cougr` issues](https://github.com/salazarsebas/Cougr/issues)

---

## What this guide will cover

Developers coming from traditional game development or web backends often hit the same wall: *what logic actually needs to be on-chain, and what doesn't?* Getting this wrong is expensive — literally, in gas fees.

This guide will answer:

- Which game state **must** live in Soroban contract storage vs. what can stay off-chain
- How Cougr's observed components (`impl_component_observed!`) bridge the gap with real-time events
- Patterns for off-chain movement with on-chain settlement
- Resource cost intuition: what makes a transaction cheap vs. expensive

Check back soon — or [watch the repository](https://github.com/salazarsebas/Cougr) to be notified when this page goes live.
