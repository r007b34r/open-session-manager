# sugyan/claude-code-webui

- Canonical URL: https://github.com/sugyan/claude-code-webui
- License: MIT
- Review Status: screened
- Reviewed At: 2026-03-16
- Absorption Mode: candidate-absorb

## Summary

Lightweight Claude browser shell with NDJSON streaming backend, project history loading, plan-mode approval UX, and single-binary distribution.

## Why It Matters To OSM

Useful clean-room reference for lightweight remote browser access, permission-mode UI, session continuity, and single-binary backend packaging.

## Project Shape

- Remote Claude browser shell
- Stack Signals:
- Hono streaming backend
- React permission-mode frontend
- single-binary backend packaging

## Verified Paths

- backend/handlers/chat.ts
- backend/history/conversationLoader.ts
- frontend/src/hooks/useClaudeStreaming.ts
- frontend/src/components/chat/PlanPermissionInputPanel.tsx

## Inspection Targets

- streaming chat transport
- history file loader
- permission and plan approval UI
- single-binary packaging

## Integration Targets

- future remote access shell
- plan approval interaction patterns
- conversation history API

## Adopted Capabilities

- None

## Upstream Source Files

- backend/handlers/chat.ts
- backend/history/conversationLoader.ts
- frontend/src/hooks/useClaudeStreaming.ts
- frontend/src/components/chat/PlanPermissionInputPanel.tsx

## Constraints

- None

## Release Acknowledgement

Added a concrete reference for lightweight remote access, session continuity, and browser-native approval UX.

## Evidence

- [Repository](https://github.com/sugyan/claude-code-webui)
