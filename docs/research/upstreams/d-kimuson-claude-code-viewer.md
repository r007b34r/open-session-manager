# d-kimuson/claude-code-viewer

- Canonical URL: https://github.com/d-kimuson/claude-code-viewer
- License: MIT
- Review Status: adopted
- Reviewed At: 2026-03-15
- Absorption Mode: candidate-absorb

## Summary

Specialized viewer for Claude Code with strong transcript fidelity, schema validation, and deep detail panes.

## Why It Matters To OSM

Strong fit for deep transcript rendering, schema validation, and lossless detail inspection.

## Project Shape

- Single-agent web viewer
- Stack Signals:
- None

## Verified Paths

- src/routes/projects/$projectId/session.tsx
- src/lib/todo-viewer/extractLatestTodos.ts
- src/server/core/claude-code/functions/parseJsonl.ts
- src/server/core/session/presentation/SessionController.ts

## Inspection Targets

- None

## Integration Targets

- None

## Adopted Capabilities

- Viewer-style transcript detail panel
- Session todo evidence in detail view

## Upstream Source Files

- src/routes/projects/$projectId/session.tsx
- src/lib/todo-viewer/extractLatestTodos.ts
- src/server/core/claude-code/functions/parseJsonl.ts
- src/server/core/session/presentation/SessionController.ts

## Constraints

- None

## Release Acknowledgement

Helped shape OSM's viewer-style transcript detail panels and todo evidence workflow.

## Evidence

- [Repository](https://github.com/d-kimuson/claude-code-viewer)
