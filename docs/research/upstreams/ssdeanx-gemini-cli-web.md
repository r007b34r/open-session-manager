# ssdeanx/Gemini-CLI-Web

- Canonical URL: https://github.com/ssdeanx/Gemini-CLI-Web
- License: LicenseRef-Conflicting
- Review Status: screened
- Reviewed At: 2026-03-16
- Absorption Mode: reference-only

## Summary

Responsive Gemini CLI web workspace with auth, WebSocket chat, file and git explorers, shell panel, and spec-design tooling.

## Why It Matters To OSM

Strong reference for remote Gemini workspace features, auth, WebSocket streaming, spec workflows, and integrated file/git/shell panels, but the license posture is currently contradictory.

## Project Shape

- Remote Gemini web workspace
- Stack Signals:
- React and Express web workspace
- JWT auth with WebSocket streaming
- embedded editor, git, shell, and spec tooling

## Verified Paths

- server/index.js
- server/sessionManager.js
- documentation/API/openapi.yaml
- src/components/SpecDesign/SpecDesign.jsx

## Inspection Targets

- session persistence model
- remote auth and WebSocket flows
- spec generation surface
- integrated shell and git workspace

## Integration Targets

- remote governance backlog
- future OpenAPI surface
- spec workflow references

## Adopted Capabilities

- None

## Upstream Source Files

- server/index.js
- server/sessionManager.js
- documentation/API/openapi.yaml
- src/components/ChatInterface.jsx
- src/components/SpecDesign/SpecDesign.jsx
- src/components/Shell.jsx

## Constraints

- Do not copy code into OSM while the license conflict remains unresolved.
- conflicting license metadata between LICENSE, README, and package metadata
- public distribution risk until the upstream clarifies reuse terms

## Release Acknowledgement

Raised the bar for Gemini-specific remote workspace breadth, but remains reference-only until licensing is clear.

## Evidence

- [Repository](https://github.com/ssdeanx/Gemini-CLI-Web)
