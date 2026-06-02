<script>
  import { onMount, onDestroy } from 'svelte';
  import { createSessionWs } from '../lib/api.js';

  let events = [];
  let connected = false;
  let reconnecting = false;
  let ws = null;
  let reconnectTimer = null;
  let reconnectDelay = 1000;
  let idCounter = 0;
  let expandedSession = null;

  const MAX_EVENTS = 1000;

  // Group raw events into sessions (bounded by session-start events)
  $: sessions = buildSessions(events);

  function buildSessions(evts) {
    if (evts.length === 0) return [];

    const groups = [];
    let current = null;

    // Events arrive newest-first, so iterate in reverse to build chronologically
    for (let i = evts.length - 1; i >= 0; i--) {
      const ev = evts[i];
      if (ev.hook === 'session-start') {
        current = {
          startTs: ev.ts,
          sessionId: ev.session_id || null,
          prompts: 0,
          toolCalls: 0,
          domains: new Set(),
          rulesInjected: 0,
          suppressed: 0,
          errors: 0,
          events: [],
          maxPromptNum: 0,
        };
        groups.push(current);
      }
      if (!current) {
        current = {
          startTs: ev.ts,
          sessionId: ev.session_id || null,
          prompts: 0,
          toolCalls: 0,
          domains: new Set(),
          rulesInjected: 0,
          suppressed: 0,
          errors: 0,
          events: [],
          maxPromptNum: 0,
        };
        groups.push(current);
      }
      // Capture session_id from any event if not set yet
      if (!current.sessionId && ev.session_id) {
        current.sessionId = ev.session_id;
      }

      current.events.push(ev);
      if (!ev.success) current.errors++;

      if (ev.hook === 'user-prompt-submit') {
        current.prompts++;
        current.rulesInjected += ev.rules_injected || 0;
        current.suppressed += ev.suppressed || 0;
        if (ev.prompt_num && ev.prompt_num > current.maxPromptNum) {
          current.maxPromptNum = ev.prompt_num;
        }
        if (ev.domains_matched) {
          ev.domains_matched.forEach(d => current.domains.add(d));
        }
      } else if (ev.hook === 'pre-tool-use' || ev.hook === 'post-tool-use') {
        current.toolCalls++;
      }
    }

    // Convert Sets to arrays, compute durations and brackets, reverse so newest is first
    return groups.reverse().map((s, i) => {
      const n = s.maxPromptNum;
      let bracket = null;
      if (n > 0) {
        if (n <= 3) bracket = { label: 'FRESH', color: 'var(--green)' };
        else if (n <= 10) bracket = { label: 'MODERATE', color: 'var(--primary)' };
        else if (n <= 20) bracket = { label: 'DEPLETED', color: 'var(--orange)' };
        else bracket = { label: 'CRITICAL', color: 'var(--red)' };
      }
      return {
        ...s,
        id: i,
        domains: [...s.domains],
        isLive: i === 0,
        lastTs: s.events[s.events.length - 1]?.ts || s.startTs,
        bracket,
      };
    });
  }

  function connect() {
    ws = createSessionWs();
    reconnecting = true;

    ws.onopen = () => {
      connected = true;
      reconnecting = false;
      reconnectDelay = 1000;
    };

    ws.onmessage = (evt) => {
      try {
        const raw = typeof evt.data === 'string' ? evt.data : '';
        if (!raw) return;
        const data = JSON.parse(raw);
        data._id = ++idCounter;
        events = [data, ...events].slice(0, MAX_EVENTS);
      } catch (e) {
        console.warn('[ws] parse error:', e);
      }
    };

    ws.onclose = () => {
      connected = false;
      scheduleReconnect();
    };

    ws.onerror = () => {
      connected = false;
      ws.close();
    };
  }

  function scheduleReconnect() {
    if (reconnectTimer) return;
    reconnecting = true;
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      reconnectDelay = Math.min(reconnectDelay * 2, 30000);
      connect();
    }, reconnectDelay);
  }

  function relTime(iso) {
    const diff = (Date.now() - new Date(iso).getTime()) / 1000;
    if (diff < 5) return 'just now';
    if (diff < 60) return `${Math.floor(diff)}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function duration(startIso, endIso) {
    const ms = new Date(endIso).getTime() - new Date(startIso).getTime();
    const s = Math.floor(ms / 1000);
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${Math.floor(s / 60)}m`;
    return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`;
  }

  function toggleExpand(id) {
    expandedSession = expandedSession === id ? null : id;
  }

  const hookColors = {
    'session-start': 'var(--primary)',
    'user-prompt-submit': 'var(--green)',
    'pre-tool-use': 'var(--yellow)',
    'post-tool-use': 'var(--accent-purple)',
  };
  const hookLabels = {
    'session-start': 'SESSION',
    'user-prompt-submit': 'PROMPT',
    'pre-tool-use': 'PRE-TOOL',
    'post-tool-use': 'POST-TOOL',
  };

  function shortPath(p) {
    if (!p) return '';
    // Strip home dir prefix for readability
    return p.replace(/^\/home\/[^/]+\//, '~/');
  }

  onMount(() => { connect(); });
  onDestroy(() => {
    if (ws) ws.close();
    if (reconnectTimer) clearTimeout(reconnectTimer);
  });
</script>

<div class="main-header">
  <div class="header-left">
    <h2>Session Activity</h2>
    <div class="connection-status" class:connected class:reconnecting>
      <span class="status-dot"></span>
      {#if connected}Live{:else if reconnecting}Reconnecting{:else}Offline{/if}
    </div>
  </div>
  <span class="session-count">{sessions.length} session{sessions.length !== 1 ? 's' : ''}</span>
</div>

<div class="main-content">
  {#if sessions.length === 0}
    <div class="empty-state">
      <h3>No sessions recorded</h3>
      <p>Hook events will appear here when a Claude Code session runs.</p>
    </div>
  {:else}
    <div class="session-list">
      {#each sessions as session (session.id)}
        <div class="session-card" class:live={session.isLive} class:has-errors={session.errors > 0}>
          <button class="session-summary" on:click={() => toggleExpand(session.id)}>
            <div class="session-top">
              {#if session.isLive}
                <span class="live-badge">● LIVE</span>
              {/if}
              <span class="session-time">{relTime(session.startTs)}</span>
              {#if session.sessionId}
                <span class="session-id" title={session.sessionId}>{session.sessionId.slice(0, 8)}</span>
              {/if}
              {#if session.events.length > 1}
                <span class="session-duration">{duration(session.startTs, session.lastTs)}</span>
              {/if}
              {#if session.bracket}
                <span class="bracket-badge" style="color: {session.bracket.color}; border-color: {session.bracket.color}" title="Context bracket: prompt depth {session.maxPromptNum}">{session.bracket.label}</span>
              {/if}
              {#if session.errors > 0}
                <span class="error-count">{session.errors} error{session.errors > 1 ? 's' : ''}</span>
              {/if}
              <span class="expand-icon">{expandedSession === session.id ? '▾' : '▸'}</span>
            </div>

            <div class="session-stats">
              <span class="stat">
                <strong>{session.prompts}</strong> prompt{session.prompts !== 1 ? 's' : ''}
              </span>
              <span class="stat-sep">·</span>
              <span class="stat">
                <strong>{session.toolCalls}</strong> tool call{session.toolCalls !== 1 ? 's' : ''}
              </span>
              {#if session.rulesInjected > 0}
                <span class="stat-sep">·</span>
                <span class="stat">
                  <strong>{session.rulesInjected}</strong> rules
                </span>
              {/if}
              {#if session.suppressed > 0}
                <span class="stat-sep">·</span>
                <span class="stat dedup">
                  {session.suppressed} dedup
                </span>
              {/if}
            </div>

            {#if session.domains.length > 0}
              <div class="session-domains">
                {#each session.domains as domain}
                  <span class="domain-chip">{domain}</span>
                {/each}
              </div>
            {/if}
          </button>

          {#if expandedSession === session.id}
            <div class="session-events">
              {#each [...session.events].reverse() as ev (ev._id)}
                <div class="ev-row">
                  <span class="ev-badge" style="background: {hookColors[ev.hook] || 'var(--ink-tertiary)'}">
                    {hookLabels[ev.hook] || ev.hook}
                  </span>
                  {#if ev.tool_name}
                    <span class="ev-tool">{ev.tool_name}</span>
                  {/if}
                  {#if ev.file_path}
                    <span class="ev-file">{shortPath(ev.file_path)}</span>
                  {/if}
                  {#if ev.prompt_text}
                    <span class="ev-prompt-text" title={ev.prompt_text}>{ev.prompt_text.length > 80 ? ev.prompt_text.slice(0, 77) + '…' : ev.prompt_text}</span>
                  {/if}
                  {#if ev.domains_matched && ev.domains_matched.length > 0}
                    {#each ev.domains_matched as d}
                      <span class="ev-domain">{d}</span>
                    {/each}
                    <span class="ev-meta">{ev.rules_injected} rules</span>
                  {/if}
                  {#if !ev.success}<span class="ev-err">FAIL</span>{/if}
                  <span class="ev-spacer"></span>
                  <span class="ev-ts">{relTime(ev.ts)}</span>
                  {#if ev.hook === 'user-prompt-submit' && ev.prompt_num}
                    <span class="ev-prompt">#{ev.prompt_num}</span>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .main-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    border-bottom: 1px solid var(--border);
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .main-header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--ink-primary);
  }
  .connection-status {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--ink-tertiary);
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--red);
  }
  .connection-status.connected .status-dot { background: var(--green); }
  .connection-status.reconnecting .status-dot {
    background: var(--yellow);
    animation: pulse 1s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .bracket-badge {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.5px;
    padding: 1px 7px;
    border-radius: 4px;
    border: 1px solid;
  }
  .session-count {
    font-size: 11px;
    color: var(--ink-tertiary);
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px 24px;
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--ink-tertiary);
  }
  .empty-state h3 { margin: 0 0 4px; font-size: 15px; font-weight: 500; }
  .empty-state p { margin: 0; font-size: 12px; }

  .session-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .session-card {
    background: var(--surface-02);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    transition: border-color 0.15s;
  }
  .session-card:hover {
    border-color: var(--border-hover, #3a3c40);
  }
  .session-card.live {
    border-color: var(--green);
    border-width: 1px;
  }
  .session-card.has-errors {
    border-color: rgba(255, 77, 77, 0.4);
  }

  .session-summary {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: inherit;
    padding: 12px 16px;
    cursor: pointer;
    font-family: inherit;
  }
  .session-summary:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .session-top {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .live-badge {
    font-size: 10px;
    font-weight: 700;
    color: var(--green);
    letter-spacing: 0.5px;
  }
  .session-time {
    font-size: 12px;
    color: var(--ink-secondary);
    font-weight: 500;
  }
  .session-id {
    font-size: 10px;
    color: var(--ink-tertiary);
    font-family: monospace;
    background: var(--surface-03);
    padding: 1px 5px;
    border-radius: 3px;
    cursor: help;
  }
  .session-duration {
    font-size: 11px;
    color: var(--ink-tertiary);
    background: var(--surface-03);
    padding: 1px 6px;
    border-radius: 4px;
  }
  .error-count {
    font-size: 10px;
    color: var(--red);
    font-weight: 600;
  }
  .expand-icon {
    margin-left: auto;
    font-size: 11px;
    color: var(--ink-tertiary);
  }

  .session-stats {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 6px;
  }
  .stat {
    font-size: 12px;
    color: var(--ink-secondary);
  }
  .stat strong {
    color: var(--ink-primary);
    font-weight: 600;
  }
  .stat.dedup {
    color: var(--ink-tertiary);
  }
  .stat-sep {
    font-size: 12px;
    color: var(--ink-tertiary);
  }

  .session-domains {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .domain-chip {
    font-size: 10px;
    color: var(--accent-cyan);
    background: rgba(0, 200, 255, 0.07);
    padding: 1px 7px;
    border-radius: 8px;
    border: 1px solid rgba(0, 200, 255, 0.12);
  }

  /* Expanded event list */
  .session-events {
    border-top: 1px solid var(--border);
    padding: 6px 12px;
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 300px;
    overflow-y: auto;
  }
  .ev-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    border-radius: 3px;
  }
  .ev-row:hover {
    background: rgba(255, 255, 255, 0.02);
  }
  .ev-badge {
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.3px;
    padding: 1px 4px;
    border-radius: 2px;
    color: var(--canvas);
    flex-shrink: 0;
  }
  .ev-tool {
    font-size: 9px;
    color: var(--ink-secondary);
    font-weight: 500;
  }
  .ev-file {
    font-size: 9px;
    color: var(--ink-tertiary);
    font-family: monospace;
  }
  .ev-prompt-text {
    font-size: 10px;
    color: var(--ink-muted);
    font-style: italic;
    max-width: 500px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: help;
  }
  .ev-domain {
    font-size: 9px;
    color: var(--accent-cyan);
    opacity: 0.7;
  }
  .ev-meta {
    font-size: 9px;
    color: var(--ink-tertiary);
  }
  .ev-err {
    font-size: 8px;
    font-weight: 700;
    color: var(--red);
  }
  .ev-spacer { flex: 1; }
  .ev-ts {
    font-size: 9px;
    color: var(--ink-tertiary);
    font-variant-numeric: tabular-nums;
  }
  .ev-prompt {
    font-size: 9px;
    color: var(--ink-secondary);
    background: var(--surface-03);
    padding: 0 3px;
    border-radius: 2px;
  }
</style>
