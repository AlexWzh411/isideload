# HANDOFF — one-click Apple Watch companion install physically validated

Date: 2026-09-09

## Objective

Record the exact end-to-end physical result for the Windows iLoader + isideload Apple Watch companion installation chantier, including the failure progression that led to the final fix.

## Repository / branch

- repository: `Rzbck/isideload`
- local worktree: `E:\_Project\IOS APP\_Tools\iloader-watch\isideload`
- branch: `feat/watch-companion-support-20260909`
- physically validated code SHA: `9d43554571360cb27c701efbe1fbd1f5456769ae`
- commit: `fix(watch): refresh companion proxy per forward`
- CI run for this SHA: `34405740552`
- CI result: SUCCESS on Windows, macOS and Ubuntu.

This file is a final physical-validation supplement. Do not replace the code SHA above with this documentation commit when describing the binary that was actually tested.

## Exact iLoader chain used for the physical success

- iLoader repository: `Rzbck/iloader`
- iLoader branch: `feat/watch-companion-support-20260909`
- iLoader code SHA: `88ca24bbb6fd028f4f180a5f28a5683fba10e7f5`
- pinned isideload rev: `9d43554571360cb27c701efbe1fbd1f5456769ae`
- iLoader workflow run: `34406032515`
- first Windows attempt failed after successful compilation only while Tauri downloaded WiX: `os error 10054` / remote host forcibly closed the connection;
- rerun of the failed jobs on the SAME iLoader SHA succeeded;
- Windows artifact name: `windows-exe`;
- artifact ID: `10125812841`;
- artifact ZIP digest: `sha256:739826aa9404bda38d666f6e10c7e0b1690b7d63918053d40e0f16bf5672da74`;
- exact local installer SHA-256: `4FE492056689602C9F02A35763959A14E11A522562825990C579C9390A74AEB9`.

Regression IPA used throughout the final transport tests:

`WatchSensorLab-companion-unsigned-f539fe4105df.ipa`

## Physical result — SUCCESS

The user physically confirmed the final exact chain above works end to end:

- the iPhone application installs successfully;
- the embedded Apple Watch companion installs successfully;
- the Apple Watch app works correctly on the real Watch;
- the one-click iLoader companion-install path is therefore physically validated for this known-good regression IPA.

Do not weaken this statement to CI-only validation: this result is based on the user's real iPhone + Apple Watch observation.

## Failure progression that matters

Earlier physical attempts proved the following sequence:

1. A previous one-click build failed at the first Watch `StartForwardingServicePort` call with `device socket io failed`.
2. The code was changed so the CompanionProxy used before the iPhone install was dropped and a fresh CompanionProxy was opened after the iPhone install.
3. That moved the physical flow forward enough for the Apple Watch itself to display a trust/pairing request.
4. The user initially denied the trust request accidentally; isideload correctly returned `user denied pairing trust`.
5. After restarting the Watch and accepting the trust request, pairing succeeded and the flow advanced further.
6. The next failure was `Failed to forward Apple Watch installation proxy` with `device socket io failed` at the second CompanionProxy forwarding command.
7. Comparison with the already-working pymobiledevice3 path showed that pymobiledevice3 opens a fresh `com.apple.companion_proxy` lockdown service connection for each forwarding start/stop command.
8. isideload SHA `9d435545...` adopted that lifecycle: fresh CompanionProxy service connection for every Watch forward start/stop.
9. The exact iLoader build pinning this SHA then physically succeeded end to end.

This strongly supports persistent CompanionProxy service reuse as the final transport-lifetime blocker. It does not mean earlier provisioning/signing/routing fixes were unnecessary; they remain part of the physically validated chain.

## Earlier corrections that remain valid

Do not regress these while refactoring:

- paired Apple Watch discovery/registration;
- correct Watch developer-services routing using the `ios/` URL path with `DTDK_Platform=watchos`;
- explicit signing of embedded `Watch/*.app` bundles because the signing dependency does not recurse into `Watch/`;
- Watch bundle ID and `WKCompanionAppBundleIdentifier` rewriting;
- split provisioning/signing for iPhone and Watch bundles;
- direct Watch `streaming_zip_conduit` install implementation;
- iLoader preservation of the exact selected usbmux device/transport;
- the direct Python Watch install proved that the generated Watch profile/signature were acceptable on the real Watch even though the profile `Platform` array did not literally contain `watchOS`.

Do not return to the disproved assumption that the provisioning profile must list `watchOS` in its `Platform` array.

Do not resume Watch syslog work; that path produced no usable diagnostics and is no longer needed for this solved blocker.

## Validation boundary

Physically validated:

- one-click iLoader install of the known-good `f539fe...` iPhone + embedded Watch IPA;
- iPhone installation;
- Watch installation;
- Watch application operation on the real device.

Not implied by this result:

- every future Watch app or entitlement will install without additional work;
- tracker/product branch behavior is validated;
- HealthKit/GPS/WatchConnectivity product integration is validated.

## Next step

The tooling regression gate is cleared. Preserve this exact chain while the application chantier moves to `Rzbck/ios-godot-lab` branch `feat/watch-sensor-tracker-recorder-20260909` for its own build and physical validation.

Do not merge to `main`, publish a release, or change generic signing/provisioning behavior without separate evidence and explicit user approval.
