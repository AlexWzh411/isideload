# HANDOFF — isideload Watch companion support

Date: 2026-09-09

## Objective

Add first-class support for embedded watchOS companion apps stored under `Main.app/Watch/*.app`, so iLoader can re-sign an iPhone app and its Watch companion coherently and install both from Windows.

## Repository state

- repository: `Rzbck/isideload`
- upstream: `nab138/isideload`
- branch: `feat/watch-companion-support-20260909`
- base SHA: `3d42025ecac97a2548d5b88aefc8028307e369c1`
- local path: `E:\_Project\IOS APP\_Tools\iloader-watch\isideload`
- latest code commit pushed for automatic Watch installation: `fef906387b0b4f2959bdea0e0c103bdcf0df21c6`
- message: `feat(watch): install companions via streaming zip conduit`

Always verify branch, HEAD, status and remote before further local writes. Local worktree was fast-forwarded to `fef906387b0b4f2959bdea0e0c103bdcf0df21c6` on 2026-09-09, but local Rust tests did NOT run because `cargo` is not installed/in PATH on that Windows environment.

## What is physically proven

Using the exact Watch bundle signed by the patched iLoader/isideload path:

- rewritten iPhone and Watch bundle IDs are coherent;
- `WKCompanionAppBundleIdentifier` matches the rewritten iPhone ID;
- `WKApplication = true`;
- Watch `embedded.mobileprovision`, `_CodeSignature/CodeResources` and signed executable are present;
- the paired Watch UDID is present in the embedded provisioning profile;
- direct Watch `installation_proxy` access works;
- `com.apple.afc` is unavailable on the Watch;
- `com.apple.streaming_zip_conduit` is available and SSL-enabled.

A stale Watch placeholder/coordinator was removed successfully with Watch `installation_proxy`:

```text
UNINSTALL: SUCCESS
BUNDLE ABSENT: OUI
```

The exact signed Watch app was then streamed directly to the real Watch through `com.apple.streaming_zip_conduit`.

Physical watchOS result:

```text
CreatingStagingDirectory 5%
ExtractingPackage 15%
InspectingPackage 20%
PreflightingApplication 30%
VerifyingApplication 40%
CreatingContainer 50%
InstallingApplication 60%
PostflightingApplication 70%
SandboxingApplication 80%
GeneratingApplicationMap 90%
InstallComplete 100%
Status: DataComplete
```

Afterward:

```text
SequenceNumber: 1516
IsPlaceholder: absent / no longer true
```

The user then put the Watch back on and launched **Watch Sensor Lab successfully on the physical Apple Watch**.

Classification: **WATCH INSTALL + LAUNCH PHYSICALLY VALIDATED for the exact signed bundle and manual direct-install path.**

This proves the signed Watch bundle/profile are accepted by watchOS. The previous profile `Platform = ['iOS', 'xrOS', 'visionOS']` observation is therefore NOT the install blocker for this bundle.

## Actual remaining root cause

The normal iPhone install path can create/update a Watch placeholder/coordinated install but does not finalize the Watch app in this environment.

The working path is:

```text
signed Watch .app
  -> paired Watch lockdown via CompanionProxy
  -> remove stale placeholder/install if present
  -> com.apple.streaming_zip_conduit
  -> watchOS InstallComplete / DataComplete
```

Therefore the remaining tooling fix is to automate this proven direct Watch installation path after the iPhone app install and before temporary signed-bundle cleanup.

## Current automation patch

Code SHA:

`fef906387b0b4f2959bdea0e0c103bdcf0df21c6`

Main changes:

- add `isideload/src/sideload/watch_install.rs`;
- retain/use the signed `Watch/*.app` before cleanup;
- pair/connect to the Watch through `CompanionProxy` forwarding;
- best-effort uninstall existing Watch app/placeholder by bundle ID;
- stream the signed Watch app with the same `streaming_zip_conduit` protocol proven physically;
- wait for watchOS progress/completion;
- integrate this after the existing iPhone install in `Sideloader::install_app()`;
- enable the required `idevice` `pair` feature.

## Validation state of `fef9063...`

NOT YET VALIDATED:

- `cargo fmt --check` — not run locally (`cargo` unavailable);
- `cargo test -p isideload --lib` — not run locally;
- `cargo build -p minimal` — not run locally;
- GitHub Actions — no workflow run existed yet for this SHA at the last check;
- iLoader pin/build — not done yet;
- one-click iLoader physical install of iPhone + Watch — not done yet.

Do not call `fef9063...` working until CI/build and physical iLoader validation pass.

## Next exact step

1. launch `Build isideload` manually for branch `feat/watch-companion-support-20260909` because the branch name contains `/` and the existing push trigger may not create a run;
2. verify exact run `head_sha = fef906387b0b4f2959bdea0e0c103bdcf0df21c6`;
3. fix any compile/format/test errors with normal follow-up commits;
4. when CI is acceptable, pin the exact validated isideload SHA into `Rzbck/iloader` branch `feat/watch-companion-support-20260909`;
5. build exact-SHA Windows iLoader;
6. install that iLoader and test the normal one-click flow on the same iPhone + Apple Watch;
7. only then classify automatic Watch installation as fixed.

## Do not modify

- do not hard-code Team ID or WatchSensorLab-specific identifiers;
- do not modify `Rzbck/ios-godot-lab` code from this tooling chantier unless explicitly requested;
- do not merge upstream/main or publish a release without explicit approval;
- do not conflate CI success with physical Watch validation;
- do not discard local work with destructive Git commands.
