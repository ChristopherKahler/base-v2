# BASE Multi-Tool Hook Bible

> Extracted 2026-06-04. Sources: official docs, GitHub repos, community deep dives.
> Credibility: all primary sources (official docs + source code).

## Purpose

BASE needs four hook points to function: **SessionStart**, **UserPromptSubmit** (CARL domain matching), **PreToolUse** (file path injection, grep intercept), and **PostToolUse** (timestamp updates). This document maps each tool's equivalent events, stdin/stdout contracts, and adapter requirements.

---

## 1. Event Name Mapping

| BASE Hook | Claude Code | Codex CLI | Antigravity/Gemini | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| **SessionStart** | `session_start` | `SessionStart` | `SessionStart` | ❌ None | `session.created` (plugin event) | ❌ None |
| **UserPromptSubmit** | `user_prompt_submit` | `UserPromptSubmit` | `BeforeAgent` | `beforeSubmitPrompt` ⚠️ | `experimental.chat.system.transform` ⚠️ | `pre_user_prompt` ⚠️ |
| **PreToolUse** | `pre_tool_use` | `PreToolUse` | `BeforeTool` | `beforeShellExecution` / `beforeMCPExecution` / `beforeReadFile` | `tool.execute.before` (plugin) | `pre_run_command` / `pre_read_code` / `pre_write_code` / `pre_mcp_tool_use` |
| **PostToolUse** | `post_tool_use` | `PostToolUse` | `AfterTool` | `afterFileEdit` ⚠️ | `tool.execute.after` (plugin) | `post_run_command` / `post_read_code` / `post_write_code` / `post_mcp_tool_use` |

### ⚠️ Gaps and Limitations

| Tool | Gap | Impact | Workaround |
|---|---|---|---|
| **Cursor** | `beforeSubmitPrompt` stdout is **ignored** — cannot inject context | CARL domain injection won't work via hooks | Use `.cursor/rules/*.mdc` with glob patterns instead |
| **Cursor** | No `SessionStart` event | Session init context won't auto-load | Use `alwaysApply: true` rules |
| **Cursor** | `afterFileEdit` is fire-and-forget (stdout ignored) | PostToolUse can't inject context back | Logging only — no functional impact for BASE |
| **Windsurf** | `pre_user_prompt` has **no structured stdout** — exit 0 or exit 2 only | CARL can't inject context on prompt submit | Use `.windsurf/rules/*.md` with `trigger: glob` |
| **Windsurf** | No `SessionStart` event | Session init won't auto-load | Use `trigger: always_on` rules |
| **Windsurf** | All hooks are approve/reject only — no JSON response | No context injection via any hook | Rules system is the only path |
| **OpenCode** | `experimental.chat.system.transform` has known bug (#17100) — mutations may be silently discarded | UserPromptSubmit injection unreliable | Use plugin `chat.message` hook as fallback |
| **OpenCode** | Plugins are JS/TS in-process, not shell commands | Can't call `base` binary directly — needs wrapper | Thin JS wrapper that shells out to `base` |

---

## 2. Adapter Architecture Per Tool

### Tier 1: Shell-command hooks, structured JSON stdout (drop-in)

#### Codex CLI
- **Config:** `~/.codex/hooks.json` or `<repo>/.codex/hooks.json`
- **Contract:** JSON on stdin → JSON on stdout. `additionalContext` injects into conversation. `decision: "block"` gates actions.
- **Context injection key:** `hookSpecificOutput.additionalContext` (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse)
- **Exit codes:** 0=success, 1=error (continue), 2=block (stderr=reason)
- **Instructions file:** `AGENTS.md` (same discovery pattern as CLAUDE.md)
- **Adapter work:** Event name translation only. Stdin schema nearly identical (has `tool_name`, `tool_input`, `prompt`, `cwd`). Stdout uses `hookSpecificOutput.additionalContext` instead of plain stdout print.

#### Antigravity / Gemini CLI
- **Config:** `.gemini/settings.json` or `~/.gemini/settings.json` `hooks` key, or extension `hooks/hooks.json`
- **Contract:** JSON on stdin → JSON on stdout. `hookSpecificOutput.additionalContext` injects context.
- **Context injection key:** `hookSpecificOutput.additionalContext` (SessionStart, BeforeAgent, AfterTool)
- **Exit codes:** 0=success, 2=system block (stderr=reason), other=warning (continue)
- **Instructions file:** `GEMINI.md` (hierarchical loading)
- **Adapter work:** Event name translation. `BeforeAgent` maps to UserPromptSubmit (receives `prompt` field). `BeforeTool` receives `tool_name` + `tool_input` + optional `mcp_context`. Stdout format identical pattern to Codex.

### Tier 2: Shell-command hooks, no structured stdout (rules-based)

#### Cursor
- **Config:** `.cursor/hooks.json`
- **Contract:** JSON on stdin → JSON on stdout for `beforeShellExecution`/`beforeMCPExecution`/`beforeReadFile` only. `beforeSubmitPrompt`/`afterFileEdit`/`stop` stdout is **ignored**.
- **Context injection key:** `agentMessage` field (only on interactive hooks). For prompt-level injection, use `.cursor/rules/*.mdc` files.
- **Exit codes:** 0=success (parse stdout JSON), non-zero=error (continue, don't parse)
- **Instructions file:** `.cursor/rules/*.mdc` (YAML frontmatter with `globs`, `alwaysApply`, `description`)
- **Adapter work:** PreToolUse works via `beforeShellExecution`/`beforeMCPExecution` → return `agentMessage`. UserPromptSubmit/SessionStart require generating `.mdc` rule files instead of hook output. Hybrid adapter: hooks for pre-tool, rules for prompt-level.

#### Windsurf / Devin Desktop
- **Config:** `.windsurf/hooks.json`
- **Contract:** JSON on stdin → **no structured stdout**. Hooks are approve/reject only (exit 0 vs exit 2). Stderr on exit 2 = reason shown to agent.
- **Context injection:** Not possible via hooks. Must use Rules system: `.windsurf/rules/*.md` or `.devin/rules/*.md` with YAML frontmatter (`trigger: glob`, `trigger: always_on`, `trigger: model_decision`).
- **Exit codes:** 0=allow, 2=block (stderr=reason), other=error (continue)
- **Instructions file:** `.windsurfrules` (legacy) or `.windsurf/rules/*.md` or `.devin/rules/*.md`
- **Adapter work:** `base install --windsurf` generates rule files from graph data instead of using hooks. Pre-tool hooks can still gate/audit via exit codes. Context injection entirely through rules regeneration on `base sync`.

### Tier 3: In-process JS/TS plugins (thin wrapper)

#### OpenCode
- **Config:** `opencode.json` `plugin` array or `.opencode/plugins/*.ts`
- **Contract:** TypeScript plugin functions. `tool.execute.before` receives `{tool, sessionID, callID}` + mutable `{args}`. `experimental.chat.system.transform` receives `{sessionID, model}` + mutable `{system: string[]}`.
- **Context injection:** Push strings to `output.system[]` in `experimental.chat.system.transform`. Mutate `args` in `tool.execute.before`.
- **Exit codes:** N/A — in-process. Throw to block.
- **Instructions file:** `AGENTS.md` (same pattern as Codex)
- **Adapter work:** Write a `base-opencode-plugin.ts` (~50 lines) that shells out to `base hook session-start`, `base hook user-prompt-submit`, `base hook pre-tool-use`, `base hook post-tool-use` with JSON on stdin and reads stdout. Plugin registers handlers for `event` (session.created), `experimental.chat.system.transform`, `tool.execute.before`, `tool.execute.after`.

---

## 3. Stdin Field Mapping (BASE ← each tool)

### SessionStart

| BASE Field | Claude Code | Codex | Antigravity | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| `session_id` | `session_id` | `session_id` | `session_id` | N/A | `event.sessionID` | N/A |
| `cwd` | `cwd` | `cwd` | `cwd` | N/A | `ctx.directory` | N/A |
| `source` | — | `source` (startup/resume/clear) | `source` (startup/resume/clear) | N/A | `event.type` = `session.created` | N/A |

### UserPromptSubmit

| BASE Field | Claude Code | Codex | Antigravity | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| `prompt` | `prompt` | `prompt` | `prompt` (via BeforeAgent) | `prompt` ⚠️ stdout ignored | N/A (no user message in hook input) | `tool_info.user_prompt` ⚠️ no stdout |
| `cwd` | `cwd` | `cwd` | `cwd` | `workspace_roots[0]` | `ctx.directory` | inferred from workspace |

### PreToolUse

| BASE Field | Claude Code | Codex | Antigravity | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| `tool_name` | `tool_name` | `tool_name` | `tool_name` | `tool_name` (MCP) / inferred (shell) | `tool` | `mcp_tool_name` / inferred |
| `tool_input` | `tool_input` | `tool_input` | `tool_input` | `tool_input` (MCP) / `command` (shell) | `args` (mutable) | `mcp_tool_arguments` / `command_line` |
| `file_path` | `tool_input.file_path` | `tool_input.file_path` | `tool_input.file_path` | `file_path` (beforeReadFile) | `args.file_path` | `tool_info.file_path` |
| `cwd` | `cwd` | `cwd` | `cwd` | `workspace_roots[0]` | `ctx.directory` | `tool_info.cwd` |

### PostToolUse

| BASE Field | Claude Code | Codex | Antigravity | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| `tool_name` | `tool_name` | `tool_name` | `tool_name` | inferred from `afterFileEdit` | `tool` | inferred from post_* event |
| `tool_response` | — | `tool_response` | `tool_response` | `edits[]` (afterFileEdit only) | `output` | `mcp_result` (post_mcp only) |
| `file_path` | `tool_input.file_path` | `tool_input.file_path` | `tool_input.file_path` | `file_path` | from original args | `tool_info.file_path` |

---

## 4. Stdout / Context Injection Mapping

| BASE Output | Claude Code | Codex | Antigravity | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| Print to stdout (context injection) | ✅ stdout text → system reminder | `hookSpecificOutput.additionalContext` | `hookSpecificOutput.additionalContext` | `agentMessage` (interactive hooks only) | `output.system.push(...)` | ❌ Not possible via hooks |
| Block action | exit 1 | `decision: "block"` or exit 2 | `decision: "deny"` or exit 2 | `permission: "deny"` (JSON, exit 0) | `throw new Error()` | exit 2 |

---

## 5. Config File Mapping

| Purpose | Claude Code | Codex | Antigravity | Cursor | OpenCode | Windsurf |
|---|---|---|---|---|---|---|
| **Hook config** | `~/.claude/settings.json` | `~/.codex/hooks.json` | `~/.gemini/settings.json` | `.cursor/hooks.json` | `.opencode/plugins/*.ts` | `.windsurf/hooks.json` |
| **Instructions** | `CLAUDE.md` | `AGENTS.md` | `GEMINI.md` | `.cursor/rules/*.mdc` | `AGENTS.md` | `.windsurfrules` / `.windsurf/rules/*.md` |
| **MCP servers** | `settings.json` mcpServers | `config.toml` [mcp_servers] | `mcp_config.json` | `.cursor/mcp.json` | `opencode.json` mcp | `mcp_config.json` |

---

## 6. `base install --{tool}` Implementation Spec

### `base install --codex`
1. Write `~/.codex/hooks.json` with 4 hooks mapping to `base hook {event}` commands
2. Stdout contract: return `{"hookSpecificOutput": {"hookEventName": "...", "additionalContext": "<base output>"}}`
3. Copy `CLAUDE.md` patterns to `AGENTS.md` if not present

### `base install --antigravity`
1. Write `~/.gemini/settings.json` hooks section with 4 hooks mapping to `base hook {event}`
2. Same stdout contract as Codex (nearly identical)
3. Copy instruction patterns to `GEMINI.md` if not present

### `base install --cursor`
1. Write `.cursor/hooks.json` with 3 hooks (beforeShellExecution, beforeMCPExecution, beforeReadFile)
2. Generate `.cursor/rules/base-context.mdc` with `alwaysApply: true` for session-start context
3. Generate glob-scoped `.mdc` rules from domain path triggers for prompt-level injection
4. `base sync --cursor-rules` command to regenerate rules from graph

### `base install --opencode`
1. Write `.opencode/plugins/base-plugin.ts` — thin wrapper that shells out to `base hook {event}`
2. Register handlers for `event`, `experimental.chat.system.transform`, `tool.execute.before`, `tool.execute.after`
3. Copy instruction patterns to `AGENTS.md` if not present

### `base install --windsurf`
1. Write `.windsurf/hooks.json` with pre-tool hooks for audit/gating (exit 0/2 only)
2. Generate `.windsurf/rules/base-*.md` with `trigger: always_on` for session context and `trigger: glob` for domain rules
3. `base sync --windsurf-rules` command to regenerate rules from graph
4. No context injection via hooks — rules are the sole mechanism

---

## 7. Adapter Complexity Ranking

| Tool | Effort | Why |
|---|---|---|
| **Codex** | Low | Nearly 1:1 contract. Event name + stdout key translation only. |
| **Antigravity** | Low | Same shell-command pattern. Slightly different event names. |
| **Cursor** | Medium | Pre-tool works via hooks. Prompt/session injection requires rule file generation — hybrid approach. |
| **OpenCode** | Medium | Requires JS/TS plugin wrapper. Experimental hooks may break. |
| **Windsurf** | Medium-High | No context injection via hooks at all. Full rules-based approach. Requires `base sync --windsurf-rules` to regenerate rules from graph on every sync. |
