# AP Mode Network Fallback Design

## Goal

Add an Android AP-mode path where the phone connects to the camera-created Wi-Fi AP, keeps the Camera Connector receiver reachable on that Wi-Fi link, and preserves the app's own internet access through cellular when the device and ROM allow it.

The first AP-mode slice must be useful without a built-in VPN. It should detect the actual network state, bind Camera Connector's own outbound internet work to cellular, keep camera-local work on Wi-Fi, and show clear recovery guidance when a ROM disables or deprioritizes the no-internet Wi-Fi network.

## Context

The current product route is push-based FTP/SFTP. AP mode still means the camera creates its own Wi-Fi AP and the phone joins it. Camera Connector already has:

- A Rust FTP/SFTP receiver and runtime status model in `core`.
- Android foreground service ownership for long-running receiver lifecycle.
- Android gateway boundaries around native core calls.
- A dashboard model that can show receiver status, connected devices, transfers, and failures.

AP mode should extend the Android network orchestration around the existing receiver. It should not change transfer semantics, account handling, flat output behavior, transfer logs, or project asset grouping.

## Decision

Build AP mode in two layers:

1. **AP Mode v1, no VPN.** Camera Connector explicitly observes Wi-Fi and cellular networks, starts the receiver on the Wi-Fi/AP path, and binds its own internet-facing requests to cellular. This is the default product path.
2. **AP Mode v2, optional VPN split routing.** A later experimental mode may use Android `VpnService` or an embedded TUN engine to route other apps' traffic through cellular while preserving camera IP traffic on Wi-Fi. This must remain optional because it adds user consent, privacy, policy, battery, compatibility, and existing-VPN conflicts.

The v1 product promise is intentionally scoped:

- Camera transfers should work while connected to the camera AP.
- Camera Connector should prefer cellular for its own internet work when cellular is available.
- Other apps are not forcibly rerouted in v1.
- The UI should explain when the ROM or device cannot keep both paths usable.

## Why ROM Behavior Needs Fallbacks

A camera AP usually has no internet. Android and OEM ROMs may treat that Wi-Fi network differently:

- The default network may stay on Wi-Fi, switch to cellular, or change after validation fails.
- Some ROMs show a "Wi-Fi has no internet" confirmation before keeping the link.
- Smart network switching features may disconnect the camera AP or silently prefer cellular.
- Local-only Wi-Fi concurrency depends on device support.
- VPN split routing cannot help if the ROM has already disconnected Wi-Fi or hidden cellular as a usable `Network`.

Therefore AP mode must be capability-driven at runtime instead of assuming one routing model.

## Architecture

Add Android-side orchestration around the existing core:

- `ApModeCoordinator`: owns AP-mode state, entry/exit, capability probes, and user-facing diagnostics.
- `AndroidNetworkGateway`: wraps `ConnectivityManager`, `WifiManager`, and active `Network` callbacks.
- `NetworkBindingGateway`: provides explicit execution helpers for cellular-bound HTTP/DNS work and Wi-Fi-bound camera-local probes.
- `ApReceiverController`: starts/stops `ReceiverForegroundService` with AP-mode-aware bind host, advertised host, and state paths.
- `ApModeState`: a Compose-facing read model surfaced through the existing UI layer.

The Rust core remains the source of truth for receiver behavior. Android supplies network facts and receiver settings; core continues to accept uploads and emit status/logs.

## Runtime State Model

AP mode should classify the current device state into explicit states:

- `Idle`: AP mode is not active.
- `ConnectingWifi`: user is connecting to the camera AP or Android is resolving the Wi-Fi network.
- `Ready`: Wi-Fi camera path is reachable and cellular internet is validated or not required.
- `ReceiverOnly`: camera Wi-Fi is reachable, but cellular internet is missing or unvalidated.
- `InternetOnly`: cellular works, but camera Wi-Fi is disconnected or the camera IP is unreachable.
- `BlockedBySystem`: Android/ROM is preventing the no-internet Wi-Fi link from staying connected.
- `Unsupported`: required APIs or device concurrency support are missing for the selected AP workflow.
- `Error`: an unexpected platform or receiver failure occurred.

Each state should carry diagnostics:

- Wi-Fi network present.
- Wi-Fi SSID or redacted AP label when available.
- Phone Wi-Fi IP and receiver port.
- Camera IP or expected camera subnet.
- Camera reachability result.
- Cellular network present.
- Cellular validated internet result.
- Current default network type.
- Local-only concurrency support when queryable.
- Recommended user action.

## Network Discovery And Probing

When entering AP mode:

1. Ask the user to connect to the camera AP or launch the relevant Android Wi-Fi picker/settings flow.
2. Register `ConnectivityManager.NetworkCallback` observers for Wi-Fi and cellular.
3. Identify the Wi-Fi `Network` associated with the camera AP when possible.
4. Identify a cellular `Network` with internet capability.
5. Probe the camera path through Wi-Fi:
   - Check the phone's Wi-Fi interface address.
   - Probe the configured camera IP or likely AP gateway.
   - Prefer non-invasive TCP/FTP control probes over broad subnet scans.
6. Probe cellular internet through cellular:
   - Use the cellular `Network` for a small reachability check.
   - Treat Android `NET_CAPABILITY_VALIDATED` as the strongest signal.
7. Start the receiver only after the Wi-Fi path is usable, or show a receiver-only warning if internet is missing but camera transfer can proceed.

The probe layer must avoid binding the whole process permanently unless the operation is known to be safe. Per-operation binding through `Network` APIs is preferred.

## Receiver Binding Policy

The receiver should keep the existing runtime model but accept AP-mode overrides:

- `bind_host`: prefer the Wi-Fi/AP interface address when known; fall back to `0.0.0.0`.
- `advertised_host`: set to the phone's Wi-Fi/AP address shown to the camera.
- `protocol`: reuse saved FTP/SFTP setting.
- `port`: reuse saved AP-mode or receiver setting.
- `state_dir`: app-private state.
- `output_dir`: current app-private output location until SAF/MediaStore write backend lands.

The UI must show the exact camera-facing endpoint: protocol, Wi-Fi IP, port, username, password status, and storage label.

## App Internet Policy

Camera Connector's own public internet operations should use cellular when AP mode is active and cellular is available. Examples include docs/help links, compatibility uploads in the future, update checks, or remote metadata.

Rules:

- Camera-local probes and receiver-related sockets use Wi-Fi.
- Public internet calls use cellular through a cellular `Network`.
- DNS for public internet calls must resolve through cellular, not through the camera AP.
- If cellular is unavailable, the app should degrade gracefully and keep receiver transfer available.
- The app should never promise that other apps remain online in v1.

## User Recovery Guidance

The AP-mode screen should make system behavior visible instead of presenting opaque failures.

When Wi-Fi has no internet and Android asks for confirmation:

- Tell the user to keep the camera AP connected.
- Keep polling for the Wi-Fi network after the user returns.

When Wi-Fi disappears after connection:

- Suggest disabling smart network switching, WLAN assistant, dual-channel acceleration, or similar OEM features.
- Offer to reopen Wi-Fi settings.

When cellular is missing or unvalidated:

- Show that receiving from the camera can still work.
- Mark internet-dependent app features as temporarily offline.

When camera IP is unreachable:

- Show the expected phone Wi-Fi IP and receiver endpoint.
- Ask the user to confirm the camera upload profile host, port, protocol, username, and passive FTP mode.

When AP mode cannot keep Wi-Fi and cellular available together:

- Fall back to receiver-only mode.
- Do not start VPN mode automatically.

## Optional VPN Split Routing

VPN split routing is a later feature, not part of v1. If added, it should be an explicit advanced mode with clear consent and a separate diagnostics panel.

Design constraints:

- Use Android `VpnService` and system consent.
- Detect and report existing active VPN conflicts.
- Exclude Camera Connector itself from the VPN tunnel when possible so the receiver and native core are not captured by their own tunnel.
- Route camera IP or camera AP subnet to Wi-Fi.
- Route all other traffic to cellular.
- Ensure outbound tunnel sockets are protected with `VpnService.protect(...)` to avoid VPN recursion.
- Bind protected outbound sockets to the intended `Network`.
- Keep DNS for non-camera traffic on cellular.
- Prefer Android route exclusion APIs where available; lower API levels may require a full TUN forwarding engine.

Embedded mihomo, sing-box, or tun2socks can be evaluated as the TUN engine, but the Android integration points above are the hard requirements. A rule file alone is not sufficient.

## Privacy And Policy

AP Mode v1 does not inspect or reroute other apps' traffic.

If VPN mode is later added:

- The app must clearly disclose what traffic is routed.
- The app must not collect packet contents.
- The app must explain that Android allows only one active VPN per profile.
- Google Play VPN policy review risk must be handled before release-channel enablement.
- Enterprise, side-loaded, or lab builds may expose VPN mode before public release builds.

## Testing Strategy

Unit tests:

- State reducer transitions for Wi-Fi/cellular/camera probe combinations.
- Receiver settings generation from AP network facts.
- User guidance messages for blocked/degraded states.
- Network binding gateway behavior with fake `Network` handles.

Android integration tests:

- Foreground service still starts/stops with AP-mode receiver settings.
- Compose renders `Ready`, `ReceiverOnly`, `InternetOnly`, and `BlockedBySystem` states.
- Dashboard polling continues while AP mode is active.

Device smoke tests:

- Real camera AP upload with cellular enabled.
- Real camera AP upload with cellular disabled.
- Wi-Fi no-internet confirmation flow.
- OEM smart-switching behavior on at least Samsung, Xiaomi/Redmi, Oppo/OnePlus, Vivo, Pixel, and Huawei/Honor if available.
- FTP RAW/JPEG pair upload and transfer-log validation.
- SFTP validation only after FTP AP mode is stable.

VPN experiments, if built:

- Existing VPN conflict.
- API-level behavior below and above route-exclusion support.
- DNS leak and recursion checks.
- Battery and long-upload stability.

## Acceptance Criteria

AP Mode v1 is accepted when:

- The app can guide the user onto a camera AP and show the phone's camera-facing IP.
- The receiver starts with a camera-facing endpoint that the camera can upload to.
- FTP upload from a real camera AP completes and appears in the existing transfer log and project dashboard.
- The app detects whether cellular internet is available while connected to the camera AP.
- Public app operations can be explicitly executed over cellular when cellular is available.
- The UI distinguishes camera-transfer availability from internet availability.
- The UI gives specific recovery guidance when ROM behavior disconnects or deprioritizes the no-internet Wi-Fi network.
- No VPN permission is requested in the default AP-mode flow.

## Non-Goals

- Replacing the Rust receiver.
- Reintroducing PTP/IP pull import.
- Forcing all other apps to use cellular in v1.
- Guaranteeing Wi-Fi plus cellular concurrency on every Android device.
- Implementing VPN split routing before AP-mode FTP upload is validated with real cameras.
