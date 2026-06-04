# BASE v2 — Global Rule Injection Test

**Date:** 2026-06-04
**Context:** Validates fix for two bugs: (1) GLOBAL rules not injecting via hooks, (2) `base rule` commands failing from bare directories. This test runs on a fresh Windows install after uninstall + reinstall.

---

## Pre-test: Uninstall + Reinstall

Run these steps before handing off to the test session.

### Step 1: Uninstall current BASE

```powershell
# From any directory — uninstall with full purge
base uninstall --purge
```

**Expected output:**
```
═══════════════════════════════════════
BASE v2 — Uninstall
═══════════════════════════════════════

1. Remove hooks from settings.json ... ✓
2. Remove CLAUDE.md section ... ✓ (or "no section found")
3. Remove binary ... ✓ removed C:\Users\Chris\.local\bin\base.exe
4. Purge global tier ... ✓ removed C:\Users\Chris\.base-gbl

═══════════════════════════════════════
✓ Uninstall complete
═══════════════════════════════════════
```

### Step 2: Verify clean slate

```powershell
# Should fail — binary removed
base --version

# Should not exist
dir ~\.base-gbl

# settings.json should have no base hooks
type ~\.claude\settings.json | findstr "base hook"
```

### Step 3: Reinstall via npx

```powershell
npx chrisai
```

**Note:** This will download the CURRENT GitHub Release binary (v0.1.1). The fix is NOT in that binary yet — the fix was just built on Linux. To test the fix, you need to either:

**Option A: Push a new tag and wait for CI**
```bash
# On Linux (dev machine):
git add -A && git commit -m "fix: global graph loading in hooks + --global rule flag"
git tag v0.1.2
git push origin main --tags
# Wait ~12 min for CI to build all platforms
# Then reinstall on Windows: npx chrisai@latest
```

**Option B: Copy the binary manually**
```bash
# On Linux: cross-compile for Windows
cargo build --release --target x86_64-pc-windows-gnu
# Or: copy target/release/base to Windows via shared folder
# Then on Windows: copy to C:\Users\Chris\.local\bin\base.exe
# Then run: base install --full
```

**Option C: Build from source on Windows**
```powershell
# On Windows with Rust installed:
cd <path-to-base-v2-source>
git pull
cargo build --release
copy target\release\base.exe ~\.local\bin\base.exe
base install --full
```

### Step 4: Verify install

```powershell
base --version
# Should show version

dir ~\.base-gbl
# Should exist with base.toml, domains.toml, .base/, etc.

dir ~\.base-gbl\.base\graph.trig
# Should exist — this is the global graph
```

---

## Test Script (for Claude in Windows session)

Copy everything below this line into the Windows Claude Code session as the first prompt.

---

## BEGIN TEST INSTRUCTIONS

You are running a validation test on BASE v2's global rule injection system. This test was written by the dev session that shipped the fix. Follow each step exactly, report results in the table format shown, and do NOT investigate or fix anything — observation only.

### What was fixed

1. **Hook injection** — hooks now load BOTH the global graph (`~/.base-gbl/.base/graph.trig`) AND the workspace graph into a merged Oxigraph store. Previously only the workspace graph was loaded, so GLOBAL rules (stored in the global graph) were never injected.

2. **Rule CLI** — `base rule` now accepts `--global` / `-g` flag to target `~/.base-gbl/` from any directory. Previously required being inside a directory with `.base/`.

### Test 1: Verify global graph exists

```powershell
base --version
dir ~\.base-gbl\.base\graph.trig
```

**Expected:** Version prints. graph.trig exists.

### Test 2: List existing global rules

```powershell
base rule --global list --domain GLOBAL
```

**Expected:** Shows at least 1 rule (MOP rule from install). If you see a "TEST RULE" from prior testing, note it but proceed.

### Test 3: Add a test rule from bare directory

```powershell
cd ~
base rule --global add --domain GLOBAL --text "VALIDATION RULE: If you can see this, global injection is working. Safe to remove. Added 2026-06-04."
base rule --global list --domain GLOBAL
```

**Expected:** Rule added successfully. List shows MOP rule + the new validation rule.

### Test 4: Verify hook injection (the critical test)

Close this Claude Code session and open a NEW one. On the very first prompt of the new session, type exactly:

```
hello
```

Then check the hook output that appears. Look for:

1. `[DOMAIN: GLOBAL]` header in the injected context
2. The MOP rule text
3. The VALIDATION RULE text you just added
4. DEVMODE block showing `GLOBAL` in LOADED DOMAINS with rule count ≥ 2

### Test 5: Verify --global remove

```powershell
base rule --global list --domain GLOBAL
```

Note the index of the VALIDATION RULE, then:

```powershell
base rule --global remove --domain GLOBAL --index <N>
base rule --global list --domain GLOBAL
```

**Expected:** Rule removed. Only MOP rule remains.

### Test 6: Error message without --global

```powershell
cd ~
base rule list --domain GLOBAL
```

**Expected:** Error message containing "Use --global for global rules, or run `base scaffold` to create a workspace."

### Results Table

Fill in and report:

```
| Test | Result | Notes |
|------|--------|-------|
| 1. Global graph exists | | |
| 2. List global rules | | |
| 3. Add rule from bare dir | | |
| 4. Hook injection (new session) | | |
| 5. Remove rule | | |
| 6. Helpful error message | | |
```

### If Test 4 fails

If the hook still shows empty LOADED DOMAINS after a fresh session:

1. Check which binary is running: `base --version` and `where base`
2. Check if the binary was actually updated (file date): `dir ~\.local\bin\base.exe`
3. Check hook wiring: `type ~\.claude\settings.json | findstr "base hook"`
4. Report all three — the fix may not be in the installed binary yet

### Cleanup

After all tests pass, remove the test rule if it's still there:

```powershell
base rule --global list --domain GLOBAL
# If VALIDATION RULE exists, remove it:
base rule --global remove --domain GLOBAL --index <N>
```

## END TEST INSTRUCTIONS
