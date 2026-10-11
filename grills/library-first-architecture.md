# Feature: Library-First Architecture

## Status
- **Done**

## Summary
Refactor devptp so high-level daemon/network operations are exposed as typed library APIs, while the CLI and IPC protocol remain compatible frontends over that library.

## Problem
The project already contains library modules, but the operational API is currently coupled to daemon internals, IPC dispatch, and CLI/JSON presentation. Future frontends such as a Ratatui UI and an Android client need a reusable, representation-independent API without breaking the existing CLI or Lua integration.

## Scope
### In scope
- Define a stable, high-level Rust API for starting and controlling devptp.
- Separate core daemon lifecycle and operations from IPC transport and frontend presentation.
- Keep IPC as the primary external workflow.
- Allow a frontend in the same process to start/control the daemon while also starting its IPC server when needed.
- Refactor the CLI to use the library/IPC boundary rather than owning networking behavior.
- Preserve current CLI commands, IPC methods, request/response shapes, and behavior.
- Return typed status/results and structured library errors.
- Add tests for the library API and compatibility of the IPC adapter.

### Out of scope
- Implementing the Ratatui frontend.
- Implementing an Android/mobile frontend or FFI bindings.
- Changing the underlying iroh networking or forwarding protocol.
- Redesigning the existing IPC wire format.
- Adding new user-facing commands or networking capabilities beyond what is needed for the separation.

## Inputs
- **Trigger:** Rust callers, the existing CLI, or existing IPC clients invoke daemon operations.
- **Data:** Typed Rust inputs such as endpoint tickets and `u16` ports; IPC continues to accept the current newline-delimited JSON requests.
- **Source:** Library callers provide typed values directly; IPC continues to deserialize requests; the CLI continues to parse command-line arguments.

## Outputs
- **User-visible:** Existing CLI JSON responses and command behavior remain unchanged. Future frontends receive typed library results and choose their own rendering.
- **Side effects:** Start/stop the iroh endpoint, router, peer and port-management tasks; optionally start/stop the local IPC listener.
- **Errors:** Library operations return the existing error concept, improved where necessary for stable matching/display. IPC maps errors to the existing `command_failed`/`invalid_request` responses.

## Constraints
- Performance: Preserve current asynchronous task model and avoid unnecessary serialization for same-process callers.
- Error handling: No panics for expected operational failures; preserve useful error context and existing IPC error behavior.
- Security: Preserve current per-user IPC socket behavior and Unix `0600` permissions.
- Compatibility: Existing CLI commands, IPC method names, JSON fields, response shapes, and Lua integrations must continue to work.
- Platform: Keep current Linux-only serving restriction and ensure the core API remains usable by future non-CLI frontends, including Android where supported by dependencies.

## Edge Cases
- Starting a second daemon or IPC listener when the socket is already owned must return a clear error.
- A same-process frontend must be able to use the typed API without being forced through JSON, while external clients continue using IPC.
- Shutting down must stop networking, background tasks, and IPC cleanly and be safe when called during partial startup.
- Repeated start-serving, connect, disconnect, and shutdown calls must retain current validation and failure behavior.
- IPC serialization changes must not accidentally alter optional fields, enum names, or response compatibility.
- Platform-specific serving functionality must continue to fail explicitly on unsupported platforms.

## Acceptance Criteria
- [ ] A documented high-level library entry point can start a daemon/runtime and expose typed operations for connect, serving, ping, status, port exposure, disconnect, and shutdown.
- [ ] IPC startup is an explicit/composable part of the library lifecycle, while the default CLI daemon flow still starts it.
- [ ] The CLI behavior and all documented IPC requests/responses remain compatible with current integrations.
- [ ] Core operations no longer print presentation output; CLI/IPC layers own formatting and logging.
- [ ] Typed status and result models are serializable where needed but usable without JSON by Rust frontends.
- [ ] Existing tests pass, and new tests cover lifecycle, operation delegation, error mapping, and representative IPC compatibility cases.
- [ ] Documentation explains the library/frontend boundary and how an in-process frontend can run the daemon with IPC.

## Implementation Plan

### Phase 1 — Setup
- [x] Inventory public versus internal modules and identify all CLI, IPC, and daemon responsibilities.
- [x] Define the public API shape: runtime/daemon builder, lifecycle handle, operation methods, typed result/status models, and error contract.
- [ ] Decide module visibility and naming so implementation details (`Peer`, managers, protocol internals) are not required by frontends.
- [x] Add or update library-facing documentation and API examples.

### Phase 2 — Core logic
- [x] Extract daemon creation/startup from the unconditional IPC startup path.
- [x] Provide explicit lifecycle methods/options for starting the core runtime and attaching the IPC server, including same-process use.
- [x] Move high-level operations behind the public library handle and make their results presentation-independent.
- [x] Remove `println!`/frontend-oriented output from core operations; route diagnostics through errors or appropriate logging boundaries.
- [x] Keep networking, peer, port-manager, and platform-specific behavior unchanged.

### Phase 3 — Frontends and compatibility
- [x] Refactor IPC dispatch to call the typed library operations and translate typed results/errors into the existing JSON response schema.
- [x] Refactor `Client`/CLI integration so command parsing and output remain frontend responsibilities.
- [x] Preserve IPC socket setup, permissions, request sequencing, method names, and response fields for Lua and other integrations.
- [x] Update daemon-mode CLI startup to use the new composable lifecycle API.
- [x] Document the recommended pattern for Ratatui and future in-process frontends, without implementing them.

### Phase 4 — Testing
- [x] Add unit tests for IPC request/response and wire-format compatibility.
- [ ] Add unit tests for typed operation validation and lifecycle transitions.
- [x] Add tests for startup/shutdown and partial-startup cleanup.
- [x] Add IPC wire compatibility tests covering representative success/error responses.
- [x] Verify CLI behavior and run workspace formatting, clippy, and tests in the devcontainer.
- [x] Verify supported platform gating through the Linux test/build path; cross-platform builds remain a separate release check.

## Notes
- The existing `src/lib.rs` is a module export surface, not yet a deliberate frontend-oriented API boundary.
- Keep the IPC protocol as an adapter rather than making the typed library API depend on JSON.
- Android support should be validated during implementation, but mobile-specific transport/UI/FFI work belongs to a later feature.
