# Feature: Tunnel Protocol Definition with Echo Option

## Status
- **Approved**

## Summary
Add a tunnel protocol definition to rdevptp's receiver with an echo option for testing bidirectional data transfer before replacing it with actual port tunneling.

## Problem
The receiver currently only accepts `iroh_ping::ALPN`. We need a custom tunnel protocol that will eventually forward arbitrary TCP streams between client and server. For now, the echo option lets us verify the protocol works end-to-end.

## Scope
### In scope
- Define a tunnel ALPN identifier (`"tunnel"`)
- Add a tunnel router to the receiver alongside ping (or replace ping for now)
- Implement an echo handler for the tunnel protocol
- Update the sender CLI to use the tunnel protocol

### Out of scope
- Actual port tunneling logic (future feature)
- Authentication/security
- Connection management / reconnection
- Large payload streaming (add later)

## Inputs
- **Trigger:** Client connects to the receiver using the tunnel ALPN
- **Data:** Arbitrary binary payload (for echo: any bytes; for tunnel: raw TCP stream)
- **Source:** Sender CLI via iroh endpoint connection

## Outputs
- **User-visible:** Echo response shows the echoed payload back to the sender
- **Side effects:** None beyond the protocol response
- **Errors:** Protocol handler panic drops the connection

## Constraints
- Performance: Keep protocol frame overhead minimal for future tunnel use
- Error handling: Panicking handlers drop connections (acceptable for now)
- Security: Not yet — auth comes later

## Edge Cases
- Large payloads: TBD — add message framing later if needed
- Concurrent connections: TBD — accept multiple clients later
- Shutdown: TBD — graceful disconnect later

## Acceptance Criteria
- [x] Tunnel ALPN identifier defined (`b"tunnel"`) and used in receiver
- [x] Echo handler implemented for tunnel protocol
- [x] Sender CLI connects via tunnel ALPN and triggers echo
- [x] Echo response is received and printed by sender (via `--echo` flag)

## Implementation Plan

### Phase 1 — Protocol definition
- [x] Define `TUNNEL_ALPN` constant (`b"tunnel"`) in rdevptp
- [x] Add tunnel router to receiver alongside ping (both coexist)
- [x] Create `Tunnel` struct implementing `iroh::protocol::ProtocolHandler`

### Phase 2 — Echo handler
- [x] Accept incoming connections via `ProtocolHandler::accept`
- [x] Read all data from client via `recv.read_to_end(usize::MAX)`
- [x] Echo back the same data via `send.write_all(&buf)`
- [x] Handle connection lifecycle (finish, close)

### Phase 3 — Sender integration
- [x] Update sender CLI with `run_tunnel_sender()` function
- [x] Add `--echo` flag to switch between ping and tunnel modes
- [x] Connect using tunnel ALPN and send test payload

## Notes
- This is the foundation for future port tunneling. The echo handler will be replaced with a tunnel handler that forwards raw streams.
- The tunnel and ping protocols coexist on the same endpoint — the receiver accepts both ALPNs.
- The sender CLI uses `--echo` flag to switch between ping mode (default) and tunnel/echo mode.

## Testing
```bash
# Terminal 1: Start receiver
cargo run -p devptp receiver

# Terminal 2: Send echo via tunnel
cargo run -p devptp sender <ticket> --echo
```
