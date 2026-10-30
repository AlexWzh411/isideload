# Apple Watch companion sideloading — porting handoff

This document is the clean entry point for the generic watchOS companion support developed on `feat/watch-companion-support-20260909`.

## Scope

The implementation extends the existing iOS sideloading pipeline so an IPA that contains an embedded Apple Watch companion can be treated as a bundle graph instead of a single iPhone application.

The generic responsibilities on the isideload side are:

- discover embedded app/extension bundles that require their own App ID and provisioning profile;
- preserve or rewrite bundle relationships coherently after sideload signing changes identifiers;
- provision the physical paired Watch when a Watch bundle requires it;
- select a watchOS-compatible provisioning path for the Watch bundle;
- explicitly sign nested Watch content rather than assuming the parent iPhone signature is sufficient;
- keep capabilities such as HealthKit attached to the correct bundle during provisioning;
- expose enough companion-device context for iLoader to complete the Watch installation path.

## Validation level

The matching iLoader + isideload revisions reached a real hardware result: the iPhone app and embedded Watch companion both installed and launched from the Windows-driven one-click path.

Before presenting any particular commit as the final upstream candidate, re-check the branch HEAD, CI and the exact iLoader revision that pins it. Historical investigation commits are not automatically the desired upstream patch series.

## Public cleanup rules

Public-facing documentation and review patches must not contain:

- local workstation paths;
- test-application bundle IDs unless reduced to generic examples;
- Apple credentials, certificates, keys or provisioning profiles;
- physical-device identifiers;
- application-specific activity/Health/GPS data;
- verbose failed-hypothesis history that is not needed to understand the final implementation.

Keep comments focused on invariants and why a non-obvious branch exists. Avoid comments that merely narrate debugging history.

## Upstream preparation

1. compare this feature branch against the current upstream-compatible base;
2. identify the minimal generic source changes that produced the validated behavior;
3. separate capability/provisioning changes from transport/install changes when it makes review clearer;
4. add or retain targeted tests where practical;
5. make commit messages describe behavior, not the private test application;
6. prepare a focused pull request to the upstream isideload repository;
7. only after the backend shape is reviewable, prepare the corresponding iLoader integration PR.

Do not merge to `main`, publish a crate/release, or force-update branches without explicit approval.

## Handoff for a new conversation

Start by verifying:

- repository `Rzbck/isideload`;
- branch `feat/watch-companion-support-20260909`;
- exact current HEAD;
- git/CI state;
- the branch HANDOFF if present;
- matching `Rzbck/iloader` branch and its clean `docs/WATCH_COMPANION_PORTING_NOTES.md`.

The goal is now productization/upstream review of already-working generic Watch companion support, not further changes to the Watch Tracker application.