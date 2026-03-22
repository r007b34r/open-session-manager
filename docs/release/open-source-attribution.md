# open Session Manager Open Source Attribution

This file is generated from `third_party/upstreams/catalog.json`.

## Candidate Absorb

- ChristopherA/claude_code_tools (BSD-2-Clause-Patent)
  Posture: candidate-absorb
  Why: Pushed OSM to turn export-before-delete into an actual handoff workflow instead of a raw transcript dump.
  Adopted: Markdown session handoff export
- coder/agentapi (MIT)
  Posture: candidate-absorb
  Why: Confirmed that OSM should grow beyond a GUI into a programmatic agent control surface.
- d-kimuson/claude-code-viewer (MIT)
  Posture: candidate-absorb
  Why: Helped shape OSM's viewer-style transcript detail panels and todo evidence workflow.
  Adopted: Viewer-style transcript detail panel; Session todo evidence in detail view
- daaain/claude-code-log (MIT)
  Posture: candidate-absorb
  Why: Helped shape OSM's Markdown export and summary-first cleanup workflow.
  Adopted: Richer markdown export sections; Transcript highlight digest; Claude todo snapshot
- Dimension-AI-Technologies/Entropic (MIT)
  Posture: candidate-absorb
  Why: Validated that a cross-platform governance console has real product demand beyond a single assistant.
- endorhq/rover (Apache-2.0)
  Posture: candidate-absorb
  Why: Raised the bar for how OSM should handle isolated background task execution and iteration workflows.
- farion1231/cc-switch (MIT)
  Posture: candidate-absorb
  Why: Clarified the configuration and provider governance surface OSM still needs to absorb.
- jazzyalex/agent-sessions (MIT)
  Posture: candidate-absorb
  Why: Helped shape OSM's local-first session indexing direction and the multi-assistant adapter expansion.
  Adopted: Gemini CLI session adapter; GitHub Copilot CLI session adapter; Factory Droid session adapter; OpenClaw session adapter
- junhoyeo/tokscale (MIT)
  Posture: candidate-absorb
  Why: Established the analytics baseline OSM needs for token and cost observability.
  Adopted: Local pricing lookup, cost provenance labels, and daily usage timeline
- kbwo/ccmanager (MIT)
  Posture: candidate-absorb
  Why: Clarified the level of worktree and multi-project orchestration OSM still needs.
  Adopted: Repo-local git worktree lifecycle CLI (`create / merge / delete / recycle`)
- kevinelliott/agentpipe (MIT)
  Posture: candidate-absorb
  Why: Showed how OSM can grow toward multi-agent health checks, metrics, and richer export surfaces.
- lulu-sk/CodexFlow (Apache-2.0)
  Posture: candidate-absorb
  Why: Validated OSM's Windows and WSL-first direction for multi-assistant session management.
- pchalasani/claude-code-tools (MIT)
  Posture: candidate-absorb
  Why: Gave OSM a clearer path toward skills, hooks, repair workflows, and extensible session tooling.
- sugyan/claude-code-webui (MIT)
  Posture: candidate-absorb
  Why: Added a concrete reference for lightweight remote access, session continuity, and browser-native approval UX.
- udecode/dotai (MIT)
  Posture: candidate-absorb
  Why: Added a methodology reference for turning session knowledge into reusable workflows and skills.
- vultuk/claude-code-web (MIT)
  Posture: candidate-absorb
  Why: Added a stronger benchmark for remote browser access and multi-session persistence.
- yoavf/ai-sessions-mcp (MIT)
  Posture: candidate-absorb
  Why: Helped confirm that OSM should expose a headless session retrieval surface beyond the GUI.

## Reference Only

- autohandai/commander (LicenseRef-NoLicense)
  Posture: reference-only
  Why: Added a stronger reference point for how far OSM should push local worktree orchestration and desktop session control.
  Constraints: Treat as reference-only until licensing is clarified.; no clear open-source license grant in mirrored repository
- Dicklesworthstone/coding_agent_session_search (LicenseRef-Restricted)
  Posture: reference-only
  Why: Helped validate how much connector breadth OSM needs, but its code remains reference-only.
  Constraints: Do not copy code into the public OSM distribution.; Treat as reference-only until a clean-room replacement exists.; additional license restrictions; public distribution risk
- milisp/codexia (AGPL-3.0-only)
  Posture: reference-only
  Why: Expanded OSM's target surface for headless serving, realtime events, and integrated workspace tooling.
  Constraints: Treat as reference-only unless a clean-room implementation is maintained.; AGPL obligations in public distribution
- siteboon/claudecodeui (GPL-3.0-only)
  Posture: reference-only
  Why: Raised the bar for remote, multi-device, and plugin-friendly surfaces OSM still lacks.
  Constraints: Treat as reference-only unless a clean-room implementation is maintained.; GPL reciprocal distribution requirements
- smtg-ai/claude-squad (AGPL-3.0-only)
  Posture: reference-only
  Why: Clarified how much session control and pause/resume orchestration OSM still lacks.
  Constraints: Treat as reference-only unless a clean-room implementation is maintained.; AGPL obligations in public distribution
- ssdeanx/Gemini-CLI-Web (LicenseRef-Conflicting)
  Posture: reference-only
  Why: Raised the bar for Gemini-specific remote workspace breadth, but remains reference-only until licensing is clear.
  Constraints: Do not copy code into OSM while the license conflict remains unresolved.; conflicting license metadata between LICENSE, README, and package metadata; public distribution risk until the upstream clarifies reuse terms
