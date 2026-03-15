# daaain/claude-code-log

- Canonical URL: https://github.com/daaain/claude-code-log
- License: MIT
- Review Status: adopted
- Reviewed At: 2026-03-15
- Absorption Mode: candidate-absorb

## Summary

Export-focused toolchain for Claude Code sessions with Markdown, HTML, and smart summary workflows.

## Why It Matters To OSM

Very close to OSM's export-before-delete requirement and summary-centric cleanup flow.

## Project Shape

- CLI and export tooling
- Stack Signals:
- Markdown export
- HTML export
- smart summaries

## Verified Paths

- claude_code_log/markdown/renderer.py
- claude_code_log/parser.py
- claude_code_log/converter.py

## Inspection Targets

- markdown exporter
- summary layout
- usage rollups

## Integration Targets

- value-preserving export
- cleanup preflight summaries
- export metadata format

## Adopted Capabilities

- Richer markdown export sections
- Transcript highlight digest
- Claude todo snapshot

## Upstream Source Files

- claude_code_log/markdown/renderer.py
- claude_code_log/parser.py
- claude_code_log/converter.py

## Constraints

- None

## Release Acknowledgement

Helped shape OSM's Markdown export and summary-first cleanup workflow.

## Evidence

- [Repository](https://github.com/daaain/claude-code-log)
