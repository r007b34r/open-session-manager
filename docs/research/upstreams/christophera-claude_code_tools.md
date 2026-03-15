# ChristopherA/claude_code_tools

- Canonical URL: https://github.com/ChristopherA/claude_code_tools
- License: BSD-2-Clause-Patent
- Review Status: adopted
- Reviewed At: 2026-03-16
- Absorption Mode: candidate-absorb

## Summary

Skill and hook pack for Claude Code with session closure/resume, git worktree automation, and context statusline tooling.

## Why It Matters To OSM

Very close to OSM's export-before-delete workflow because it formalizes session closure, session resume, and git worktree handoff scripts.

## Project Shape

- Workflow skill pack
- Stack Signals:
- Claude skill and hook bundle
- shell-scripted worktree automation
- session handoff and resume workflow

## Verified Paths

- skills/session-resume/SKILL.md
- skills/session-closure/SKILL.md
- skills/git-worktree/scripts/create-worktree.sh
- tools/context-monitor/scripts/status-line.sh

## Inspection Targets

- resume artifact format
- session-end checklist
- worktree lifecycle scripts
- context budget surfacing

## Integration Targets

- Markdown session handoff export
- cleanup-before-delete checklist
- future worktree automation

## Adopted Capabilities

- Markdown session handoff export

## Upstream Source Files

- skills/session-resume/SKILL.md
- skills/session-closure/SKILL.md
- skills/git-worktree/scripts/create-worktree.sh
- tools/context-monitor/scripts/status-line.sh

## Constraints

- None

## Release Acknowledgement

Pushed OSM to turn export-before-delete into an actual handoff workflow instead of a raw transcript dump.

## Evidence

- [Repository](https://github.com/ChristopherA/claude_code_tools)
