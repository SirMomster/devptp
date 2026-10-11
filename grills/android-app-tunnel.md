# Feature: Android App Tunnel

## Status
- **In Progress**

## Summary
Bring the client-side functionality from `devptp-tui` into an Android app by reusing the Rust daemon/library core behind a native Android UI, so Android can connect to a serving devptp peer and tunnel traffic through the existing peer-to-peer transport.

## Problem
The current interactive client frontend is a Ratatui desktop terminal app. Android users need a mobile UI and Android-compatible runtime packaging to connect with a serving ticket, monitor connection state, and use devptp tunnels without a terminal.

## Scope
### In scope
- Add an Android app frontend for the connecting/client role.
- Reuse the existing Rust `Daemon`/peer/tunnel code where possible.
- Expose a mobile-safe API from Rust to Android for connect, disconnect, ping, status, traffic counters, logs/events, and shutdown.
- Provide a native Android UI equivalent to the TUI: ticket entry, connect/disconnect/reconnect, connection state, RTT, forwarded ports, available ports if provided, traffic counters/graph, bounded logs, and quit/shutdown behavior.
- Package Rust native libraries for Android targets, starting with `arm64-v8a`.
- Support running as a foreground service while connected so the tunnel remains alive when the app backgrounds.
- Preserve existing CLI/TUI/IPC behavior.

### Out of scope
- Android serving mode in the first version.
- Full-device VPN routing, `VpnService`, or system-wide traffic interception.
- Play Store polish, billing, accounts, or cloud sync.
- Replacing the Ratatui frontend.
- Changing the existing desktop IPC protocol as the primary mobile interface.

## Inputs
- **Trigger:** User opens the Android app, enters/pastes a serving ticket, and taps connect; reconnect/disconnect/ping actions are user-triggered or timer-triggered.
- **Data:** Endpoint ticket string, UI events, Android lifecycle events, periodic status/ping timers, daemon/tunnel events, traffic counters.
- **Source:** Ticket from clipboard/manual entry/deep link or QR scan in a later enhancement; runtime state from the embedded Rust daemon.

## Outputs
- **User-visible:** Android screens showing ticket entry, connection state, RTT, uptime, forwarded ports, available ports, traffic counters/graph, logs, and actionable errors.
- **Side effects:** Starts an embedded devptp client runtime, connects to the serving peer, opens local forwarding listeners as currently implemented, optionally runs a foreground notification while connected, and shuts down cleanly.
- **Errors:** Bad ticket, network unavailable, Android permission/foreground-service failure, unsupported target/API behavior, connect timeout, peer disconnect, native library load failure.

## Constraints
- Performance: UI must not block networking; native callbacks/events must be bounded; traffic graph remains rolling and memory bounded.
- Error handling: Rust errors must cross the mobile API as structured errors; Android lifecycle changes must not leak daemon tasks or duplicate connections.
- Security: Do not log full tickets; protect foreground-service notification content; avoid exposing forwarded listeners beyond Android localhost unless explicitly required.
- Compatibility: Existing Rust CLI, TUI, IPC, and desktop behavior remain unchanged; Android-specific code is additive and feature-gated where needed.
- Platform: Android builds require `aarch64-linux-android` first, then optional `armv7`, `x86_64`; avoid Linux `/proc` serving assumptions on Android.

## Edge Cases
- Empty/malformed ticket; preserve input for correction.
- App backgrounded during connect or while connected.
- Android kills the foreground service or process.
- Network switches between Wi-Fi/mobile or temporarily drops.
- Repeated connect/disconnect/reconnect taps.
- Stale async events after disconnect or service restart.
- Local port binding blocked by Android restrictions or another app.
- Native panic or Rust runtime initialization failure.
- Traffic counters reset on reconnect without graph spikes.

## Acceptance Criteria
- [ ] Android debug app builds and installs with embedded Rust native library for `arm64-v8a`.
- [ ] App starts the devptp client runtime and can shut it down cleanly.
- [ ] User can paste a serving ticket and connect to an existing Linux serving peer.
- [ ] User can disconnect, reconnect with the last ticket, and edit/replace the ticket.
- [ ] App shows connection state, RTT, uptime, forwarded ports, traffic counters, and bounded logs.
- [ ] App keeps the tunnel alive in the background using a foreground service while connected.
- [ ] Existing `devptp`, `devptp-tui`, IPC docs/API, and desktop tests continue to work.
- [ ] Android lifecycle tests/manual checks cover rotation/background/foreground/disconnect/shutdown.

## Implementation Plan

### Phase 1 — Decide mobile architecture and tunnel semantics
- [x] Confirm first milestone semantics: **local port-forwarding client** on Android, not full-device VPN.
- [x] Decide Android UI stack: Kotlin + Jetpack Compose recommended for a native app shell.
- [x] Define the initial frontend-neutral Rust API boundary around the daemon.
- [ ] Decide Rust bridge approach: UniFFI recommended for structured async-ish APIs and generated Kotlin bindings; JNI can be fallback if UniFFI clashes with current async/runtime needs.
- [ ] Define the Android API surface mirroring TUI needs: `startRuntime`, `connect(ticket)`, `disconnect`, `ping`, `status`, `traffic`, `shutdown`, and event/log stream.
- [ ] Define service lifecycle: one foreground `DevptpService` owns the Rust daemon; UI binds/observes service state.

### Phase 2 — Make the Rust core Android-linkable
- [x] Add a frontend-neutral `MobileDaemon` wrapper with lifecycle, reconnect, status, traffic, and bounded log APIs.
- [x] Add an Android `aarch64-linux-android` native-library build script and documentation.
- [x] Add Android SDK, NDK, OpenJDK, Gradle, and the Rust Android target to the devcontainer.
- [x] Build the native library and debug APK successfully in the devcontainer.
- [x] Add an initial JNI bridge exposing the mobile wrapper through JSON snapshots/results.
- [ ] Split frontend-only dependencies from the reusable library: move `ratatui`, `crossterm`, `clap`, and desktop IPC dependencies behind binary/features if needed.
- [ ] Audit dependencies for Android support, especially `iroh`, `interprocess`, terminal crates, and platform-specific socket assumptions.
- [ ] Add Android cargo targets and build script support for `aarch64-linux-android` using the Android NDK.
- [ ] Add a small `devptp_mobile` Rust crate or feature module that wraps `Daemon` and owns a Tokio runtime suitable for JNI/UniFFI calls.
- [ ] Ensure Android builds compile without serving-mode `/proc` code and without desktop IPC socket startup.

### Phase 3 — Add the mobile bridge
- [x] Implement a `MobileDaemon` handle wrapping `Arc<Daemon>` plus runtime/task management.
- [x] Implement initial JNI operations for start, snapshot, connect, disconnect, ping, and shutdown.
- [ ] Expose structured DTOs for `Status`, `Role`, `TrafficSnapshot`, log entries, and errors.
- [ ] Add an event channel for status/log/traffic updates so Android does not need to poll aggressively.
- [ ] Redact tickets in all mobile logs.
- [ ] Add cancellation/session IDs to ignore stale connect/ping/status events after disconnect.

### Phase 4 — Build the Android app shell
- [x] Create an `android/` Gradle project.
- [x] Add a minimal Compose ticket/connect/disconnect screen.
- [x] Add a foreground-service skeleton for daemon ownership.
- [ ] Add native library packaging for Rust `.so` outputs under `jniLibs` or via a Gradle Rust plugin/task.
- [ ] Implement `DevptpService` as a foreground service that loads the Rust library, starts/stops the daemon, and exposes state to UI.
- [ ] Implement Compose screens: ticket input, connected dashboard, ports, traffic graph, logs, reconnect/disconnect controls, and error banners.
- [ ] Add clipboard paste support; consider QR scan/deep link as later enhancements.
- [ ] Add required permissions: `INTERNET`, `ACCESS_NETWORK_STATE`, and foreground service permissions for the target SDK.

### Phase 5 — Android tunnel behavior
- [ ] Verify the existing connecting role works on Android: peer connection, port exposure messages, and local forwarding listeners.
- [ ] Confirm Android localhost forwarding behavior from apps/browser on the same device.
- [ ] Add clear UI copy explaining how Android users consume forwarded services, e.g. `http://127.0.0.1:<port>`.

### Phase 6 — Testing and release path
- [ ] Add Rust unit tests for the mobile wrapper state machine and stale-event handling.
- [ ] Add Android instrumentation/manual test checklist for connect, disconnect, background, process death, rotation, and network change.
- [ ] Add CI jobs or scripts for Rust Android library builds and Gradle debug APK builds.
- [ ] Document setup in `docs/android.md`: NDK install, build commands, supported ABIs, known limitations.
- [ ] Produce a debug APK and test against a Linux serving peer.

## Notes
- The current `devptp-tui` already uses `Daemon::start()`, `start_peer()`, `disconnect()`, `ping()`, `status()`, and `traffic()`; these are the core APIs to wrap for Android.
- Product decision confirmed: Android will use local forwarded ports only. Full-device VPN routing is explicitly out of scope.
- Android serving mode should remain later work because current serving/port discovery is Linux-specific.
