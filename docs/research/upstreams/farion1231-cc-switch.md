# farion1231/cc-switch

- Canonical URL: https://github.com/farion1231/cc-switch
- License: MIT
- Review Status: adopted
- Reviewed At: 2026-03-16
- Absorption Mode: candidate-absorb

## Summary

Desktop switchboard for provider presets, MCP, prompts, skills, usage, proxy failover, and session management.

## Why It Matters To OSM

Strong reference for provider, proxy, MCP, prompts, and skills governance. OSM has now started clean-room absorption on the Gemini and OpenClaw config side.

## Project Shape

- Provider governance console

## Verified Paths

- `src-tauri/src/gemini_config.rs`
- `src-tauri/src/openclaw_config.rs`
- `docs/user-manual/en/2-providers/2.2-switch.md`
- `docs/user-manual/en/5-faq/5.1-config-files.md`

## Inspection Targets

- Gemini auth mode and `.env` layout
- OpenClaw config structure and JSON5 write model
- Provider/base URL field mapping
- Managed app config file locations

## Integration Targets

- Gemini CLI config audit
- OpenClaw config audit
- Future provider/MCP governance backlog

## Adopted Capabilities

- Gemini CLI config audit
- OpenClaw config audit

## Upstream Source Files

- `src-tauri/src/gemini_config.rs`
- `src-tauri/src/openclaw_config.rs`
- `docs/user-manual/en/2-providers/2.2-switch.md`
- `docs/user-manual/en/5-faq/5.1-config-files.md`

## Constraints

- None

## Release Acknowledgement

Provided the clearest path and field model for OSM's first Gemini CLI and OpenClaw config governance pass.

## Evidence

- [Repository](https://github.com/farion1231/cc-switch)
