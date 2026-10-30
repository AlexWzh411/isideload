# HANDOFF — Apple Watch companion support in isideload

Date: **2026-09-09**

## Objective

Add generic support for embedded Apple Watch companion apps to `isideload`, so an IPA containing `Main.app/Watch/Companion.app` can be re-signed/provisioned coherently.

This is a separate tooling chantier from `Rzbck/ios-godot-lab` / Watch Sensor Lab. Do not modify the application repo from this fork unless explicitly requested.

## Repository / current branch

- repo: `Rzbck/isideload` (fork of `nab138/isideload`)
- local path: `E:\_Project\IOS APP\_Tools\iloader-watch\isideload`
- branch: `feat/watch-companion-support-20260909`
- upstream baseline used by iLoader 2.3.1: `3d42025ecac97a2548d5b88aefc8028307e369c1`
- latest code HEAD verified before this HANDOFF update: `7c8d2a3008890ac771597ef72092cc7204e93b0b`
- local worktree was verified clean at that exact SHA before this documentation update.
- after this HANDOFF commit, always re-fetch GitHub/Git before asserting current HEAD.

## Implemented Watch support so far

### Generic embedded Watch bundle support

At code SHA `04d25c73742d29c2b8936c6f95741f22640ef6da`:

- detect embedded `.app` bundles under `Watch/`;
- keep them as `watch_apps` in the bundle tree;
- rewrite Watch `CFBundleIdentifier` consistently with the re-signed iPhone app;
- rewrite `WKCompanionAppBundleIdentifier` to the rewritten main iPhone bundle ID;
- include Watch app bundles in App-ID registration/provisioning collection;
- persist rewritten `Info.plist` files recursively;
- include Watch bundles in isideload's own nested-bundle traversal;
- add a regression test for coordinated bundle-ID rewriting.

CI run `34366333338` at exact SHA `04d25c73742d29c2b8936c6f95741f22640ef6da`: **SUCCESS** on macOS, Ubuntu and Windows.

### Paired Watch discovery / provisioning attempt

At SHA `8923077eddfc6c70aa7225e2a06a744e1e0c3b0d`:

- enabled `idevice` companion proxy support;
- discovered paired Apple Watch from the connected iPhone;
- attempted registration of the paired Watch as a development device;
- routed Watch App ID/profile operations through `DeveloperDeviceType::Watchos`.

CI run `34374806678`: **SUCCESS** on all three OSes.

Physical result: failed before install with HTTP 404 on:

`https://developerservices2.apple.com/services/QH65B2/watchos/listDevices.action?...`

This proved companion discovery reached the Watch-registration path, but the QH65B2 URL routing was wrong.

### Corrected Apple Developer Services routing

At current code SHA `7c8d2a3008890ac771597ef72092cc7204e93b0b`:

- Watch operations use the `ios/` QH65B2 path;
- explicit Watch operations add `DTDK_Platform = watchos`;
- applied to device registration and App ID/profile calls;
- regression tests verify Watch uses the iOS service path and carries the Watch platform marker.

CI run `34378142101`: **COMPLETED SUCCESS** on macOS, Ubuntu and Windows.

Physical result through corrected iLoader backend:

- previous QH65B2 404 is gone: **physically validated**;
- iPhone app installs and launches: **physically validated**;
- Watch companion is detected/advertised but still fails installation with the Watch message equivalent to:
  **“Impossible d’installer Watch Sensor Lab — cette app ne peut pas être installée car son intégrité n’a pas pu être vérifiée.”**
- Watch icon is still missing; treat this as a separate asset/metadata issue until proven otherwise.

## Windows / Watch device validation completed

Using `pymobiledevice3` from Windows:

- iPhone USB communication works;
- paired Watch is visible through CompanionProxy;
- Watch lockdown forwarding/pairing works;
- Watch identified as `Watch7,14`, watchOS `26.6`;
- Watch Developer Mode option was revealed through AMFI and then manually enabled/restarted by the user.

Therefore the remaining integrity failure is **not explained by Developer Mode being absent**.

## Current signing diagnosis — IMPORTANT

A system-log capture during the real installation attempt shows:

- the rewritten iPhone bundle installs successfully;
- the rewritten Watch bundle ID is advertised as locally available:
  `com.rzbck.watchsensorlab.<TEAM_ID>.watchkitapp`;
- the Watch app does not become an installed Watch application;
- the iPhone application has its expected `embedded.mobileprovision` and valid signed entitlements during installation.

Source inspection now exposes a concrete signing-path mismatch:

1. `isideload/src/sideload/sign.rs` correctly populates `BundleSigningSettings.embedded_mobileprovisions_by_bundle_id` and `entitlements_by_bundle_id` for nested bundles.
2. `isideload` itself knows about `Watch/*.app` via its patched `watch_apps` tree.
3. However the actual `apple-codesign` implementation used by `sign_bundle()` discovers nested bundles only under:
   - `Frameworks/`
   - `PlugIns/`
4. It does **not** currently recurse into `Watch/`.

Consequences in the current code path:

- the Watch profile can be requested and stored in `all_profiles`, but the final signer never opens `Watch/<Companion>.app` as a nested bundle;
- therefore the Watch-specific `embedded.mobileprovision` / entitlements map is not applied through that recursive signing path;
- the Watch executable/bundle is not signed as a first-class nested app by `apple-codesign`;
- this matches the remaining physical “integrity could not be verified” failure.

This is now the primary root-cause candidate and is substantially stronger than the previous device-registration hypothesis. **Final confirmation still requires a code correction plus physical Watch install/launch validation.**

## Exact current stopping point

Before the next code modification, the user verified locally:

```text
branch: feat/watch-companion-support-20260909
HEAD:   7c8d2a3008890ac771597ef72092cc7204e93b0b
status: clean
```

No signing fix for `Watch/` has been applied yet.

## Next exact step

Patch the signing backend so `Watch/*.app` is treated as a nested application bundle by the actual `apple-codesign` recursion used by `sign_bundle()`.

Before choosing implementation strategy, inspect the exact pinned `Dadoum/apple-crates` revision and dependency behavior. Prefer the smallest generic fix that:

1. recursively signs `Watch/*.app`;
2. applies `embedded_mobileprovisions_by_bundle_id` to the Watch bundle ID;
3. applies Watch profile entitlements to the Watch executable;
4. preserves existing `Frameworks/` and `PlugIns/` behavior;
5. adds a regression test proving a Watch bundle gets a nested profile/signing pass;
6. passes CI on macOS, Ubuntu and Windows;
7. is pinned into iLoader;
8. is tested on the same physical iPhone + Apple Watch.

Only after physical success should this be considered fixed.

## Do not do

- do not hard-code the user's Team ID;
- do not special-case `WatchSensorLab` by name — support must stay generic;
- do not conflate CI/build success with physical Apple Watch validation;
- do not treat the missing Watch icon as the integrity root cause without evidence;
- do not merge to upstream/main automatically;
- do not modify `Rzbck/ios-godot-lab` from this tooling fork without explicit scope change;
- do not discard or overwrite local work with destructive Git commands.

## Update - explicit Watch bundle signing

Code commit:

dd4109c6ead22823f956e3f0f20d480e4b9965df
fix(watch): sign embedded watchOS app bundles

Root cause addressed:

- apple-codesign recursively discovers Frameworks/ and PlugIns/, but not Watch/.
- isideload already had the Watch provisioning profile and Watch entitlements in its per-bundle maps.
- the embedded Watch app therefore needed an explicit sign_bundle() pass before signing the iPhone root bundle.

Implementation:

for watch_app in app.bundle.watch_apps() {
    sign_bundle(&watch_app.bundle_dir, &settings)?;
}

Expected effect:

- embedded.mobileprovision inside the Watch app;
- Watch-specific entitlements;
- signed Watch executable;
- _CodeSignature for the Watch bundle.

CI validation:

- run: 34385333279
- exact code SHA: dd4109c6ead22823f956e3f0f20d480e4b9965df
- Windows: tests PASS, build PASS, artifact upload PASS.
- macOS: tests PASS, build PASS, artifact upload PASS.
- Ubuntu: infrastructure failure before tests during apt-get update.
- Ubuntu failure: Google Chrome apt repository Hash Sum mismatch.
- rerun attempt 2 reproduced the same external apt failure.

Windows isideload artifact:

- name: minimal-windows.exe
- artifact ID: 10117522030
- digest: sha256:2609a0cd4d8aaafee3c77a3aa857b63188ddee7c0313d8a4b0c84c5bd1868401

Physical Apple Watch validation of this signing fix has NOT been performed yet.

Next exact step:

1. Pin iLoader to dd4109c6ead22823f956e3f0f20d480e4b9965df.
2. Build an exact-SHA Windows iLoader.
3. Install that experimental iLoader.
4. Reinstall the SAME Watch Sensor Lab IPA.
5. Verify physical Watch installation and launch.

## Update - physical result of explicit signing patch

The explicit Watch `sign_bundle()` patch at code SHA `dd4109c6ead22823f956e3f0f20d480e4b9965df` was pinned into iLoader code SHA `045caa99d3122cd2bcba878588b1678a9cf5f6dc` and tested physically on the same real iPhone + Apple Watch.

Result:

- iPhone reinstall succeeds;
- Watch delivery is attempted;
- Watch installation DB updates/recreates the Watch bundle placeholder;
- the Watch app still does **not** finalize;
- `IsPlaceholder = True` remains.

Observed Watch DB progression during fresh attempts:

- previous `SequenceNumber = 1501`;
- explicit-signing clean reinstall: `SequenceNumber = 1505`;
- later fresh reinstall during diagnostics: `SequenceNumber = 1509`;
- container UUID and bundle path changed between attempts.

This proves the latest install attempts are reaching watchOS and refreshing the placeholder. The explicit signing patch is **necessary evidence-wise but not sufficient** to make the Watch app install.

## Update - Watch syslog path is not yielding usable live logs

Using `pymobiledevice3 11.12.0` against the paired Watch through CompanionProxy + forwarded Watch lockdown:

- Watch connection works (`Watch7,14`, watchOS `26.6`);
- `OsTraceService.get_pid_list()` succeeds and returned **358 processes**;
- relevant processes include `amfid`, `appconduitd`, `appstored`, `installcoordinationd`, `installd`, `misagent`, `securityd`;
- global `OsTraceService.syslog()` yielded no entries;
- adding `PROMISCUOUS` still yielded no entries;
- classic `SyslogService` yielded no useful lines;
- per-PID targeted capture during a fresh reinstall also wrote **0 bytes**.

Treat the live syslog route as non-productive for now. The zero-byte files contain no diagnostic evidence and should not be filtered further.

## Update - exact post-signing bundle captured and inspected

The exact signed app bundle produced by the physically tested iLoader path was captured from its temporary extraction directory before isideload cleanup removed it.

Local capture:

`E:\_Project\IOS APP\_Tools\pymobiledevice3-watch\signed-captures\WatchSensorLab-signed-20260909_205028.app`

The embedded Watch bundle contains:

- `Info.plist`;
- `embedded.mobileprovision` (**12720 bytes**);
- `_CodeSignature\CodeResources` (**2182 bytes**);
- `WatchSensorLabWatch` executable (**371759 bytes**).

Therefore the explicit Watch signing pass is definitely producing a Watch profile + bundle signature on disk. The remaining failure occurs despite those files being present.

### Captured Watch Info.plist is coherent

Observed from the exact signed bundle:

- rewritten iPhone ID: `com.rzbck.watchsensorlab.59858TV9N2`;
- rewritten Watch ID: `com.rzbck.watchsensorlab.59858TV9N2.watchkitapp`;
- `WKCompanionAppBundleIdentifier` exactly matches the rewritten iPhone ID;
- `WKApplication = true`;
- `WKRunsIndependentlyOfCompanionApp = false`;
- `CFBundleExecutable = WatchSensorLabWatch`;
- `DTPlatformName = watchos`;
- `MinimumOSVersion = 10.0`;
- `UIDeviceFamily = [4]`.

### Captured Watch provisioning profile — IMPORTANT NEW EVIDENCE

The profile embedded in the exact signed Watch app reports:

- name: `iOS Team Provisioning Profile: com.rzbck.watchsensorlab.59858TV9N2.watchkitapp`;
- `Platform = ['iOS', 'xrOS', 'visionOS']`;
- **no `watchOS` entry is present in `Platform`**;
- expiration: `2026-09-16 18:50:59`;
- `ProvisionedDevices` count: **2**;
- the physical paired Watch UDID is present in `ProvisionedDevices`;
- `application-identifier` matches the Watch bundle ID;
- team identifier matches the profile team;
- `get-task-allow = true`.

This materially changes the diagnosis:

- missing Watch registration is no longer the leading explanation because the physical Watch UDID is in the actual embedded profile;
- companion ID rewrite is correct;
- `WKApplication` is correct;
- explicit Watch signing/profile embedding is occurring;
- Developer Mode is enabled;
- yet the embedded Watch profile is still named as an **iOS Team Provisioning Profile** and its `Platform` list lacks `watchOS`.

## Current strongest root-cause candidate — NOT YET PROVEN

The strongest current evidence points at the **Watch provisioning-profile acquisition path**, not at bundle discovery or missing signing files.

Even though current isideload code routes Watch developer-service operations through the `ios/` QH65B2 service path and adds `DTDK_Platform = watchos`, the actual profile returned and embedded into the Watch bundle appears to be an iOS-family profile (`iOS/xrOS/visionOS`) rather than an explicitly watchOS profile.

Do **not** claim final root cause yet. We still need to determine whether a valid modern watchOS development profile is expected to include `watchOS` in the `Platform` array and, if yes, which request parameter/action/profile type is currently wrong.

## Next exact step after this documentation update

Before modifying code again:

1. inspect `download_team_provisioning_profile` and all Watch-specific profile creation/download calls in `isideload/src/dev/app_ids.rs` / related developer-service code;
2. compare exact request payloads for iOS vs `DeveloperDeviceType::Watchos`;
3. verify which profile type/action Apple expects for a watchOS application and whether `DTDK_Platform=watchos` alone is sufficient;
4. determine why the real returned profile has `Platform = ['iOS', 'xrOS', 'visionOS']`;
5. only then implement the smallest generic provisioning fix;
6. run tests/CI, pin exact code SHA into iLoader, and physically retest.

No new code patch has been made for this profile-platform finding yet.

## Update - physical pairing succeeded; persistent CompanionProxy reuse isolated

Date: **2026-09-09**

### Exact builds physically tested

isideload backend:

`367d24c6443897d586493128bef0210203525157`

iLoader pin/build:

`ce868720316adf1b92b4fb1083db2f230f36f7ca`

iLoader CI:

- run `34403672997`;
- completed SUCCESS;
- Windows build and Windows EXE upload SUCCESS.

Exact Windows setup SHA-256 physically used:

`390667937EC44A9D820D217183F46F6F263003E8C574B981EFC6E16CB68E58F7`

Same regression IPA:

`WatchSensorLab-companion-unsigned-f539fe4105df.ipa`

### Physical result - important progression

The iPhone application installs successfully.

The previous failure at the initial Watch lockdown forwarding step no longer occurs.

On the first attempt, the real Apple Watch displayed a pairing/trust prompt. The user accidentally denied it. isideload then correctly failed with:

`Failed to pair with the Apple Watch through companion proxy`
`user denied pairing trust`

After restarting the Watch, repeating the exact same installation and accepting the trust/pairing prompt, pairing proceeded successfully.

The installation then advanced further and failed at:

`Failed to forward Apple Watch installation proxy`

at `watch_install.rs:193`, with:

`device socket io failed`

This is materially later than the previous failure. It proves:

- initial `com.apple.mobile.lockdownd` forwarding succeeded;
- the forwarded Watch lockdown connection succeeded;
- Watch pairing/trust succeeded after user approval;
- the Watch lockdown session progressed far enough to start `com.apple.mobile.installation_proxy`;
- failure occurs when sending the NEXT `StartForwardingServicePort` command through CompanionProxy.

### Current root-cause candidate

At SHA `367d24c...`, one persistent `CompanionProxy` connection is reused across multiple forwarding operations.

The working pymobiledevice3 implementation behaves differently: every `start_forwarding_service_port()` and `stop_forwarding_service_port()` call starts a fresh `com.apple.companion_proxy` lockdown service connection before sending its command.

There is also historical libimobiledevice evidence that a CompanionProxy connection may successfully start one Watch forwarding operation and then fail on a subsequent operation with a mux/broken-pipe style error.

Therefore the next patch changes isideload to use a fresh CompanionProxy service connection for EVERY Watch forwarding start/stop command:

1. Watch lockdownd start forward;
2. Watch installation_proxy start forward;
3. installation_proxy stop forward;
4. streaming_zip_conduit start forward;
5. streaming_zip_conduit stop forward;
6. Watch lockdownd stop forward.

This is a root-cause-directed transport-lifetime correction, not another retry/timing patch.

### Validation boundary

The fresh-per-command CompanionProxy patch is NOT physically validated yet.

Do not declare the one-click Watch path fixed until:

1. isideload CI passes;
2. exact new isideload SHA is pinned into iLoader;
3. iLoader CI passes;
4. exact Windows setup is installed;
5. the SAME `f539fe...` regression IPA is used;
6. iPhone installs;
7. Watch install reaches streaming_zip_conduit and completes;
8. Watch Sensor Lab launches physically on the Apple Watch.

If the next test fails, the exact new `watch_install.rs` context/line must be recorded before changing anything else.
