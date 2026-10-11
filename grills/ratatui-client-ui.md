# Feature: Ratatui Client UI

## Status
- **In Progress**

## Summary
Add a polished cross-platform Ratatui client frontend that owns an in-process devptp daemon, connects to serving peers by ticket, displays ports/logs/connection statistics, and shuts the daemon down on exit.

## Problem
The CLI and IPC integration provide the current client workflow, but there is no interactive frontend for monitoring a connection. A TUI should make the client experience easier to operate while exposing live forwarded/available ports, diagnostics, RTT, and application traffic metrics for profiling.

## Scope
### In scope
- Add a Ratatui-based client-only binary/frontend.
- Start and own the daemon in-process using the library API.
- Provide an initial ticket input screen.
- Connect, disconnect, reconnect, and overwrite the ticket.
- Reuse the last ticket by default on reconnect.
- Display forwarded local ports and the serving peer's available ports.
- Show bounded logs from daemon/network/client activity.
- Periodically measure and display RTT.
- Track application/tunnel bytes sent and received, sampled over a two-minute graph window.
- Provide a confirmation prompt before quitting while connected.
- Shut down the owned daemon when the TUI exits.
- Support Linux and Windows builds; serving controls are not exposed.
- Include a polished layout and ASCII devptp logo.

### Out of scope
- Ratatui controls for starting a serving daemon.
- Changing the existing CLI or IPC protocol behavior.
- Showing raw iroh/QUIC overhead in traffic metrics.
- A mobile frontend.
- Port selection: available ports remain informational; existing automatic exposure behavior is retained.

## Inputs
- **Trigger:** User launches the TUI client.
- **Data:** A user-entered endpoint ticket, key events, periodic timer ticks, daemon events, peer traffic counters, and status changes.
- **Source:** Ticket input comes from the initial form or reconnect form; runtime data comes from the owned library daemon and peer/tunnel event instrumentation.

## Outputs
- **User-visible:** A responsive TUI with logo, connection state, role, uptime, RTT, forwarded ports, available ports, traffic graph, bounded logs, and actionable keybinding hints.
- **Side effects:** Starts an in-process daemon, connects to a serving peer, automatically handles newly available ports using existing behavior, periodically pings the peer, and shuts down the daemon on exit.
- **Errors:** Connection failures, invalid tickets, unavailable IPC/runtime resources, terminal setup/restore failures, and peer/tunnel errors appear in the log/status area without crashing the UI where recovery is possible.

## Constraints
- Performance: UI rendering and sampling must not block networking; retain only bounded logs and a two-minute rolling metrics history.
- Error handling: Invalid tickets and connection errors return the user to an actionable input/reconnect state. Shutdown must restore the terminal even after errors.
- Security: Preserve iroh ticket handling and current local runtime behavior; do not log full tickets by default.
- Compatibility: Existing CLI, Lua integration, and IPC wire format remain unchanged. The TUI is an additional client frontend.
- Platform: Do not depend on Unix-only terminal, socket, or `/proc` behavior; serving remains unavailable from this frontend and the TUI must compile on Windows.
- Presentation: Core library code emits structured events/counters rather than printing directly. CLI and TUI choose their own rendering/logging.

## Edge Cases
- Empty, malformed, or expired tickets must show an error and preserve the input for correction.
- Reconnect with the previous ticket should be the default, with an explicit way to replace it.
- Disconnect while a connection attempt or periodic ping is in flight must cancel/ignore stale work.
- Repeated connect, disconnect, reconnect, and quit actions must not spawn duplicate daemons, samplers, or ping tasks.
- A serving peer can have no available ports, or ports can appear/disappear while connected.
- The two-minute graph must handle startup, idle traffic, counter resets, and reconnects without misleading spikes.
- Logs may be produced faster than the UI can display them; oldest entries must be evicted at the configured bound.
- Terminal resize, unsupported color capabilities, and terminal restoration failures must be handled gracefully.
- Closing the TUI must shut down the daemon and all background tasks, including after a failed connection.

## Acceptance Criteria
- [ ] The TUI launches into a ticket input screen and can connect using the entered ticket.
- [ ] The TUI owns the in-process daemon and shuts it down on exit.
- [ ] The TUI has client-only actions: connect, disconnect, reconnect, ping/status monitoring, and ticket replacement; no serving action is exposed.
- [ ] Reconnect defaults to the previous ticket and supports replacing it.
- [ ] Forwarded ports and serving-peer available ports are both displayed and update live.
- [ ] Logs from library/network/client activity appear in a bounded, scrollable in-memory log view without exposing full tickets by default.
- [ ] RTT updates periodically and is shown in the connection summary.
- [ ] Application/tunnel bytes sent and received are tracked separately from protocol overhead and rendered over a rolling two-minute graph.
- [ ] Quit prompts for confirmation while connected and restores the terminal in all exit paths.
- [ ] The UI has a polished cross-platform layout, including a devptp ASCII logo and keybinding help.
- [ ] Existing CLI/Lua/IPC behavior remains unchanged.
- [ ] Linux and Windows builds compile successfully, and tests cover metrics, bounded logs, state transitions, and ticket/reconnect behavior.

## Implementation Plan

### Phase 1 — Frontend and event architecture
- [x] Choose the frontend binary/package layout and add Ratatui plus the minimal terminal/event dependencies.
- [x] Define a TUI application state machine for ticket entry, connecting, connected, disconnected, reconnecting, error, and quit-confirmation states.
- [ ] Define structured runtime events for logs, status/port changes, peer lifecycle, RTT samples, and application traffic samples.
- [x] Define bounded log storage and a rolling two-minute traffic sample model independent of rendering.
- [x] Define cross-platform terminal initialization, event polling, resize handling, and restoration ownership.

### Phase 2 — Library instrumentation
- [ ] Replace remaining core diagnostic printing with structured event delivery that can be consumed by the TUI without changing CLI/IPC behavior.
- [ ] Add an optional event sink/subscriber to the daemon/runtime, with no-op/default behavior for existing callers.
- [x] Instrument tunnel/application read and write paths to count bytes sent and received, explicitly excluding iroh/QUIC overhead.
- [x] Expose peer/port lifecycle updates needed to update forwarded and available port views.
- [x] Add a periodic ping/RTT operation that can be scheduled by the TUI and cancelled during disconnect/shutdown.

### Phase 3 — TUI implementation
- [x] Implement the initial ticket form, validation, connection action, and masked/safe ticket-related logging.
- [x] Implement the connected layout: ASCII logo, state/header panel, RTT and uptime, port panels, traffic graph, logs, and keybinding footer.
- [x] Implement disconnect, reconnect-with-last-ticket, and reconnect-with-edited-ticket flows.
- [x] Implement informational available-port updates while retaining existing automatic forwarding behavior.
- [x] Implement bounded log scrolling/filtering and graph scaling/idle states.
- [x] Implement quit confirmation and guaranteed daemon/terminal cleanup.

### Phase 4 — Testing and verification
- [ ] Unit-test the state machine, ticket transitions, reconnect defaults/overrides, quit confirmation, and stale async event handling.
- [ ] Unit-test bounded log eviction and two-minute traffic sampling/counter reset behavior.
- [ ] Test application byte accounting at tunnel boundaries, including empty payloads and reconnects.
- [ ] Add rendering/snapshot or deterministic view-model tests for key UI states where practical.
- [x] Run `cargo fmt`, `cargo clippy`, and workspace tests in the devcontainer.
- [ ] Verify Linux and Windows compilation and terminal cleanup paths.
- [ ] Manually verify CLI/Lua IPC compatibility remains unchanged.

## Notes
- The existing `Daemon::start()` and `Daemon::shutdown()` APIs provide the intended ownership lifecycle for the TUI.
- Available ports should remain informational because automatic exposure is already part of the client behavior.
- Metrics must be application/tunnel-level counters, not endpoint-level network statistics.
- The graph should use a rolling two-minute window and degrade cleanly before enough samples exist.
