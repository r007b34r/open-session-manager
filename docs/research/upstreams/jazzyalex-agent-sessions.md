# jazzyalex/agent-sessions

- Canonical URL: https://github.com/jazzyalex/agent-sessions
- License: MIT
- Review Status: adopted
- Reviewed At: 2026-03-16
- Absorption Mode: candidate-absorb

## Summary

Desktop-first local session explorer with deep indexing, search, analytics, and broad assistant parser coverage.

## Why It Matters To OSM

Strong alignment with OSM's local-first indexing and session analytics direction.

## Project Shape

- Desktop session explorer
- Stack Signals:
- SwiftUI macOS desktop shell
- local-first multi-agent indexers
- analytics and live cockpit surfaces

## Verified Paths

- AgentSessions/AgentSessionsApp.swift
- AgentSessions/Views/UnifiedSessionsView.swift
- AgentSessions/Analytics/Services/AnalyticsService.swift
- AgentSessions/OpenCode/OpenCodeSqliteReader.swift

## Inspection Targets

- unified session indexer
- search and transcript navigation
- analytics rollups
- active session cockpit

## Integration Targets

- adapter coverage expansion
- session search UX
- analytics dashboard roadmap
- OpenCode SQLite compatibility

## Adopted Capabilities

- Gemini CLI session adapter
- GitHub Copilot CLI session adapter
- Factory Droid session adapter
- OpenClaw session adapter

## Upstream Source Files

- AgentSessions/Views/UnifiedSessionsView.swift
- AgentSessions/Services/GeminiSessionIndexer.swift
- AgentSessions/Services/CopilotSessionIndexer.swift
- AgentSessions/Services/DroidSessionIndexer.swift
- AgentSessions/Services/OpenClawSessionIndexer.swift
- AgentSessions/Analytics/Services/AnalyticsService.swift
- AgentSessions/OpenCode/OpenCodeSqliteReader.swift

## Constraints

- None

## Release Acknowledgement

Helped shape OSM's local-first session indexing direction and the multi-assistant adapter expansion.

## Evidence

- [Repository](https://github.com/jazzyalex/agent-sessions)
