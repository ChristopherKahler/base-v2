# Claude Code Hook Configuration

Add these hooks to your `~/.claude/settings.json` to wire BASE v2 into Claude Code sessions.

## settings.json snippet

```json
{
  "hooks": {
    "SessionStart": [
      {
        "type": "command",
        "command": "base hook session-start"
      }
    ],
    "PostToolUse": [
      {
        "type": "command",
        "command": "base hook post-tool-use"
      }
    ]
  }
}
```

## How it works

### SessionStart

Fires once when a Claude Code session begins. The hook:

1. Discovers TriG graph files (global `~/.base-gbl/graph.trig` + workspace `.base/graph.trig`)
2. Loads them into an in-memory Oxigraph store
3. Runs SPARQL queries from `queries.toml` (or embedded defaults)
4. Emits formatted results to stdout → injected as a `<system-reminder>` in the session

**If no graph files exist or any error occurs:** silent exit (fail-open). No injection.

### PostToolUse

Fires after every tool use (Edit, Write, Read, Bash, etc.). The hook:

1. Extracts file paths from the event JSON (stdin)
2. Finds the workspace `.base/graph.trig`
3. Matches file paths against entities with `ops:path` set
4. Updates `ops:lastActive` timestamps via SPARQL UPDATE
5. Atomically writes the graph back

**No stdout output.** This hook only mutates the graph — it doesn't inject context.

## Prerequisites

- `base` binary on `$PATH` (or use absolute path in the command)
- At least one `.base/graph.trig` file for session-start to have data to query
- Entities need `ops:path` set for post-tool-use to match file activity

## Fail-open guarantee

Both hooks exit 0 on any error. Errors are logged to stderr only. Claude Code sessions are never blocked by a hook failure.
