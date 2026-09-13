# Feature: JSON IPC API

## Status
- **Done**

## Summary
Replace the current ad-hoc local-socket IPC protocol with a documented JSON-RPC-like request/response API that returns machine-readable JSON and exposes daemon status, connection, forwarding, and lifecycle controls.

## Problem
The daemon already communicates over a local socket using JSON internally, but responses are unstructured enum values and the CLI converts them to human-readable text. External systems therefore cannot reliably call the daemon or interpret results. The daemon should retain its networking behavior while presenting a stable, user-scoped IPC interface.

## Scope
### In scope
- Keep the local socket transport.
- Use JSON-RPC-like requests with an optional request `id`, `method`, and `params`.
- Return a consistent envelope such as `{ "id": ..., "ok": true, "result": ... }` or `{ "id": ..., "ok": false, "error": { "code": ..., "message": ... } }`.
- Support `ping`, `start_serving`, `connect`, `expose_port`, `status`, `disconnect`, `list_forwarded_ports`, and `shutdown`.
- Make the CLI print raw JSON responses.
- Allow multiple concurrent IPC clients.
- Ensure forwarded-port results contain ports detected by the daemon and permitted by configuration.
- Clean up peer/forwarding state on disconnect.
- Gracefully stop networking, port monitoring, and IPC on shutdown.
- Restrict the socket to the current OS user.
- Reject a second daemon instance and remove/recover stale socket state.

### Out of scope
- New networking or tunnel functionality.
- HTTP, stdin/stdout, or other transports.
- Compatibility with the existing IPC wire format.
- Language-specific external client libraries.
- Authentication beyond current-user local socket permissions.

## Inputs
- **Trigger:** A local process connects to the daemon socket and sends one JSON request per line/message.
- **Data:** JSON-RPC-like object with optional `id`, a method name, and method-specific params.
- **Source:** CLI or any local process running as the daemon’s OS user.

## Outputs
- **User-visible:** The CLI prints the exact JSON response; other clients receive the same response over the socket.
- **Side effects:** Commands may start serving, connect/disconnect a peer, request port exposure, or gracefully shut down the daemon.
- **Errors:** Structured error responses with stable error codes and human-readable messages; malformed/unknown requests must not panic the daemon.

## Constraints
- Performance: Preserve the existing async local-socket architecture and support concurrent client connections.
- Error handling: Every request receives a response when possible; command failures are represented as structured errors. Avoid `expect`/panic paths in IPC handling.
- Security: The local socket must be usable only by the current OS user. A second daemon must not take over an active endpoint; stale socket state should be removed safely.
- Compatibility: The old IPC format may be replaced.

## Edge Cases
- Invalid JSON, missing fields, unknown methods, invalid parameter types, and missing required parameters.
- A request for a peer-dependent operation while no peer is connected.
- Starting serving when it is already active.
- Connecting while another peer is connected.
- Disconnecting when no peer is connected.
- Listing ports before the port monitor has populated its state.
- Multiple clients issuing commands concurrently, including shutdown.
- Client disconnects while requests are being processed.
- Stale socket left after an unclean daemon exit.
- Attempting to start a second daemon while the first is active.

## Acceptance Criteria
- [x] Requests use the agreed JSON-RPC-like shape and optional IDs are echoed in responses.
- [x] All listed methods are implemented and return consistent success/error envelopes.
- [x] CLI commands output raw JSON rather than formatted status text.
- [x] `status` reports running, serving, connected, ticket, and forwarded-port state where available.
- [x] `list_forwarded_ports` returns detected ports filtered by configured allowed ports.
- [x] Disconnect clears the active peer and forwarding state.
- [x] Shutdown gracefully stops daemon tasks and networking and exits the daemon process.
- [x] Multiple IPC clients can operate concurrently.
- [x] Socket access is restricted to the current user, stale sockets are recoverable, and active duplicate daemons are rejected.
- [x] Build, unit tests, and a live status/shutdown IPC smoke test pass.

## Implementation Plan

### Phase 1 — API and lifecycle model
- [ ] Define serializable request, params, success-result, and structured-error types.
- [ ] Define stable method names and result shapes for status, port listing, connection, and lifecycle operations.
- [ ] Add shared daemon shutdown/cancellation state and explicit cleanup hooks for peer, forwarding, port-monitor, and IPC tasks.

### Phase 2 — IPC server
- [ ] Replace the existing `Request`/`Response` dispatch with JSON-RPC-like decoding and envelope encoding.
- [ ] Validate requests and return structured errors without terminating the connection or daemon.
- [ ] Implement all command handlers while preserving existing networking logic.
- [ ] Make port exposure report/consume the serving side’s available-port information rather than relying on an unnecessary synchronous completion step.
- [ ] Support multiple clients and orderly shutdown of the listener.

### Phase 3 — Socket and client behavior
- [ ] Add current-user socket permission handling.
- [ ] Detect active listeners and reject duplicate daemons.
- [ ] Remove stale socket state safely during startup/shutdown.
- [ ] Update the client and CLI to pass request IDs and print raw JSON responses.

### Phase 4 — Testing and documentation
- [ ] Add serialization and validation tests for every method and error shape.
- [ ] Add local-socket integration tests for lifecycle, concurrency, malformed input, stale sockets, and duplicate daemon startup.
- [ ] Document the request/response protocol and command examples.

## Notes
- Assumption: `status.forwarded_ports` and `list_forwarded_ports` use the same configured-allowed, currently-detected port set maintained by `PortManager`.
- Assumption: shutdown is initiated through IPC and the daemon’s main process waits on a cancellation signal rather than only Ctrl+C.
- The exact local-socket permission API and stale-socket strategy depend on the `interprocess` crate behavior on the supported platforms.
