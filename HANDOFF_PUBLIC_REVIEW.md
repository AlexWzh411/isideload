# HANDOFF — Apple Watch companion support public review

Date: 2026-09-10

## Objective

Turn the already-working generic Apple Watch companion support into a minimal, professional, upstream-reviewable contribution without mixing any application-specific code or data into isideload.

## Public coordination repository

- repository: `Rzbck/iloader-watch-companion`
- visibility: public
- default branch: `main`
- tracking issue: `#1` — `Prepare clean upstream Watch companion contribution`
- primary continuation handoff: `HANDOFF.md` in that repository

Use that repository as the public coordination/source-of-truth entry point for architecture, validation evidence, security rules, testing, implementation mapping, and upstream-review status.

## Repository / regression branch

- repository: `Rzbck/isideload`
- regression branch: `feat/watch-companion-support-20260909`
- physically used backend regression anchor: `f7b9f3da570edd6824c29680545e710846d07df5`
- CI run for anchor: `34436382782` — SUCCESS
- matching physically validated iLoader source revision: `70f37e9b4afc659ab44ec1944c034093f4cda416`
- matching iLoader CI run: `34436587217` — SUCCESS
- current branch HEAD at this checkpoint: `5fdc15fa10de927b9fcd962eba4f97bb46e3e44e`
- commits after `f7b9f3da...` are documentation-only at this checkpoint

This branch is a **regression branch**, not the future upstream PR branch. Preserve it as evidence of the working physical path.

## Physical evidence

The matching iLoader/isideload revisions were exercised on real iPhone + Apple Watch hardware through the Windows one-click path. The iPhone host installed/launched and the embedded Watch companion was provisioned, signed, installed and launched on the paired Watch.

This validates the tested path only; do not generalize to all entitlements, all IPA topologies or all watchOS versions without evidence.

## Generic backend responsibilities

The working backend changes address:

- recursive discovery of nested app-ID-bearing bundles;
- coherent identifier/companion relationship rewriting after sideload signing changes IDs;
- paired-Watch provisioning;
- device/platform typing needed for Watch profiles;
- explicit signing of Watch content;
- capability-aware provisioning, including the tested HealthKit path;
- paired-Watch installation support used by the iLoader integration.

## Why a fresh review branch is required

The historical feature branch has diverged from current `main`. Its branch-level diff includes the Watch work plus broader lockfile/dependency/auth/certificate and historical diagnostic changes accumulated during the investigation.

Do **not** open an upstream PR directly from this regression branch.

Create a fresh cleanup/review branch from the relevant current `nab138/isideload` base and port only the minimum generic Watch-support behavior that is actually required. Keep unrelated dependency/auth/certificate changes out unless a concrete technical dependency is proven.

See `docs/IMPLEMENTATION_MAP.md` in `Rzbck/iloader-watch-companion` for the mapped source areas and review boundary.

## Public sanitation rules

Do not include:

- local workstation paths;
- real device identifiers;
- Apple credentials, certificates, private keys, provisioning profiles or pairing records;
- Watch Tracker source or Health/GPS/activity data;
- tracker-specific bundle identifiers except sanitized examples;
- comments that only narrate discarded debugging hypotheses.

Source comments should explain invariants and platform constraints.

## Review plan

1. continue from `Rzbck/iloader-watch-companion` issue `#1`;
2. verify current `nab138/isideload` and fork refs;
3. read `docs/WATCH_COMPANION_PORTING_NOTES.md` and the public `docs/IMPLEMENTATION_MAP.md`;
4. preserve `f7b9f3da...` unchanged as the regression anchor;
5. create a fresh upstream-compatible cleanup/review branch;
6. port the smallest generic Watch patch series preserving the validated behavior;
7. retain/add focused tests for nested bundle discovery, relationship rewriting, platform selection, capability decisions, Watch signing/install flow;
8. run exact-SHA tests/CI;
9. inspect the final diff for secrets/private identifiers and unrelated changes;
10. open a **draft PR** to `nab138/isideload`;
11. respond to maintainer feedback without merging unless explicitly approved;
12. only after backend review shape is stable, prepare the matching iLoader PR.

## Start of next conversation

Read, in this order:

1. `HANDOFF.md` in `Rzbck/iloader-watch-companion`;
2. `docs/IMPLEMENTATION_MAP.md` there;
3. public issue `#1` there;
4. this file;
5. `docs/WATCH_COMPANION_PORTING_NOTES.md`;
6. matching iLoader `PUBLIC_REVIEW_HANDOFF.md` and `HANDOFF_ONE_CLICK_WATCH_SUCCESS.md`.

Then verify repository, upstream base, regression-branch HEAD and CI before modifying source.