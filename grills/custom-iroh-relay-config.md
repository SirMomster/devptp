# Feature: Configurable iroh relay URL

## Status
- **Done**

## Summary
Allow client and server runs to use a configured iroh relay URL, with a `--config-file <path>` override and clear errors for invalid explicitly selected config.

## Problem
The relay URL should be configurable for both client and server rather than fixed to defaults, while preserving each role's current configuration behavior.

## Scope
### In scope
- Read a relay URL from configuration for clients and servers.
- Use existing `src/config.rs` path resolution for both roles, including its project-local preference and user-level fallback; retain defaults when no config is found.
- Preserve existing configuration behavior.
- Add `--config-file <path>` to select/override the config file for either role.
- Report clear errors for missing explicitly selected files and malformed/invalid config.

### Out of scope
- Configuring settings other than the relay URL as part of this feature.
- Changing the server's existing default config location or unrelated settings.

## Inputs
- **Trigger:** Starting a client or server; optionally pass `--config-file <path>`.
- **Data:** A config file containing a relay URL, such as `relay_url = "https://relay.example.com"` in TOML.
- **Source:** The explicit `--config-file` path when provided; otherwise existing `src/config.rs` path resolution (project-local config first, then configured user-level paths).

## Outputs
- **User-visible:** Client/server use the configured relay URL; absent config/settings use existing defaults.
- **Side effects:** None beyond normal endpoint setup using the selected relay.
- **Errors:** Clear startup error for an explicitly selected missing file, malformed config, or invalid relay URL; do not silently fall back in those cases.

## Constraints
- Performance: Configuration loading is startup-only.
- Error handling: Fail clearly for invalid explicit configuration; preserve default behavior when config is absent.
- Security: No secrets are introduced by the relay URL setting.
- Compatibility: Existing server config and defaults remain supported; `--config-file` takes precedence over the normal config path for either role.

## Edge Cases
- No default client config exists: continue with existing defaults.
- Config exists but omits the relay URL: use the existing default relay behavior.
- Explicit config path is missing, unreadable, malformed, or has an invalid relay URL: return a clear error.
- Existing server config contains other settings: preserve them while loading the relay URL.

## Acceptance Criteria
- [x] Client and server use the relay URL from existing config resolution, preserving project-local preference and user-level fallback.
- [x] Defaults remain unchanged when no config is found or no relay URL is configured.
- [x] `--config-file <path>` overrides the normal config source for both client and server.
- [x] Valid relay URL is applied to endpoint configuration.
- [x] Missing explicitly selected file, malformed config, and invalid URL produce clear errors.
- [x] Tests cover config loading, override precedence, fallback, and errors.

## Implementation Plan

### Phase 1 — Setup
- [x] Inspect client/server endpoint construction and existing config/CLI parsing.
- [x] Define the relay URL field in the existing config format without disrupting current fields.

### Phase 2 — Core logic
- [x] Reuse existing config-path resolution and add shared explicit-path override behavior.
- [x] Apply the configured relay URL to client and server endpoint setup.

### Phase 3 — Edge cases & error handling
- [x] Validate relay URLs and provide contextual errors for explicit config failures.
- [x] Preserve defaults for absent optional config or absent relay setting.

### Phase 4 — Testing
- [x] Add unit/integration tests for paths, override precedence, defaults, validation, and errors.
- [x] Run relevant project tests/build.

## Notes
- `src/config.rs` currently supports `DEVPTP_CONFIG`, a project-local `.devptp.toml`, and XDG/HOME config paths, and its `Config` currently contains `shared_ports`. The user confirmed that the existing project-local preference is acceptable; clients without a project-local config should fall through to user-level config.
- `--config-file` was specified as a CLI override for both roles and should take precedence over normal config selection.
