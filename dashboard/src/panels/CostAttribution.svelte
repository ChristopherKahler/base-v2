<script>
  import { onMount } from 'svelte';
  import { getLedger, getCostSummary } from '../lib/api.js';
  import * as d3 from 'd3';

  let summary = null;
  let ledger = [];
  let loading = true;
  let selectedPhase = null;
  let chartEl;

  function fmtCost(v) {
    if (v == null) return '$0.00';
    return '$' + v.toFixed(2);
  }

  function fmtCostCompact(v) {
    if (v == null || v === 0) return '$0';
    if (v < 0.01) return '<$0.01';
    return '$' + v.toFixed(2);
  }

  function timeAgo(ts) {
    if (!ts) return '';
    const now = Date.now();
    const then = new Date(ts).getTime();
    const diff = Math.floor((now - then) / 1000);
    if (diff < 60) return 'just now';
    if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
    if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
    if (diff < 604800) return Math.floor(diff / 86400) + 'd ago';
    return new Date(ts).toLocaleDateString();
  }

  const actionColors = {
    plan: 'var(--primary)',
    apply: 'var(--green)',
    unify: 'var(--accent-purple)',
    iterate: 'var(--orange)',
    discover: 'var(--accent-cyan)',
    research: 'var(--accent-lavender)',
  };

  function actionColor(action) {
    const key = (action || '').toLowerCase();
    return actionColors[key] || 'var(--ink-tertiary)';
  }

  $: mostExpensive = summary && summary.phases.length
    ? summary.phases.reduce((a, b) => a.total_cost > b.total_cost ? a : b)
    : null;

  $: avgCost = summary && summary.total_entries > 0
    ? summary.total_cost / summary.total_entries
    : 0;

  $: sortedPhases = summary
    ? [...summary.phases].sort((a, b) => b.total_cost - a.total_cost)
    : [];

  $: maxPhaseCost = sortedPhases.length ? sortedPhases[0].total_cost : 0;

  $: recentLedger = ledger.slice(0, 20);

  function selectPhase(phase) {
    selectedPhase = selectedPhase?.phase === phase.phase ? null : phase;
  }

  function renderChart() {
    if (!chartEl || !sortedPhases.length) return;

    d3.select(chartEl).selectAll('*').remove();

    const margin = { top: 8, right: 80, bottom: 4, left: 120 };
    const barHeight = 32;
    const gap = 6;
    const width = chartEl.clientWidth;
    const height = sortedPhases.length * (barHeight + gap) + margin.top + margin.bottom;

    const svg = d3.select(chartEl)
      .append('svg')
      .attr('width', width)
      .attr('height', height);

    const maxCost = d3.max(sortedPhases, d => d.total_cost) || 1;
    const x = d3.scaleLinear()
      .domain([0, maxCost * 1.1])
      .range([0, width - margin.left - margin.right]);

    const g = svg.append('g')
      .attr('transform', `translate(${margin.left}, ${margin.top})`);

    // Phase labels (left)
    g.selectAll('.phase-label')
      .data(sortedPhases)
      .enter()
      .append('text')
      .attr('x', -8)
      .attr('y', (_, i) => i * (barHeight + gap) + barHeight / 2)
      .attr('text-anchor', 'end')
      .attr('dominant-baseline', 'middle')
      .attr('fill', '#B0B2B7')
      .style('font-size', '11px')
      .text(d => d.phase.length > 16 ? d.phase.substring(0, 16) + '…' : d.phase);

    // Gradient defs
    const defs = svg.append('defs');
    const gradient = defs.append('linearGradient')
      .attr('id', 'bar-gradient')
      .attr('x1', '0%').attr('y1', '0%')
      .attr('x2', '100%').attr('y2', '0%');
    gradient.append('stop').attr('offset', '0%').attr('stop-color', '#725EFF');
    gradient.append('stop').attr('offset', '100%').attr('stop-color', '#BF6AFB');

    // Bars
    g.selectAll('.phase-bar')
      .data(sortedPhases)
      .enter()
      .append('rect')
      .attr('class', 'phase-bar')
      .attr('x', 0)
      .attr('y', (_, i) => i * (barHeight + gap))
      .attr('width', d => Math.max(2, x(d.total_cost)))
      .attr('height', barHeight)
      .attr('rx', 4)
      .attr('fill', 'url(#bar-gradient)')
      .attr('opacity', 0.85)
      .style('cursor', 'pointer')
      .on('click', (_, d) => selectPhase(d));

    // Cost labels (right of bar)
    g.selectAll('.cost-label')
      .data(sortedPhases)
      .enter()
      .append('text')
      .attr('x', d => Math.max(2, x(d.total_cost)) + 8)
      .attr('y', (_, i) => i * (barHeight + gap) + barHeight / 2)
      .attr('dominant-baseline', 'middle')
      .attr('fill', '#ffffff')
      .style('font-size', '11px')
      .style('font-weight', '600')
      .text(d => `${fmtCost(d.total_cost)}  ·  ${d.session_count} sessions`);

    // Hover highlight
    g.selectAll('.phase-bar')
      .on('mouseenter', function() { d3.select(this).attr('opacity', 1); })
      .on('mouseleave', function() {
        d3.select(this).attr('opacity', d => selectedPhase?.phase === d.phase ? 1 : 0.85);
      });

    // Selected highlight
    if (selectedPhase) {
      g.selectAll('.phase-bar')
        .attr('opacity', d => d.phase === selectedPhase.phase ? 1 : 0.5);
    }
  }

  $: if (chartEl && sortedPhases.length) {
    renderChart();
  }

  $: if (selectedPhase) {
    renderChart();
  }

  async function loadData() {
    loading = true;
    const [s, l] = await Promise.all([getCostSummary(), getLedger()]);
    summary = s;
    ledger = l || [];
    loading = false;
  }

  onMount(() => { loadData(); });
</script>

<div class="cost-panel">
  {#if loading}
    <div class="loading">
      <div class="loading-pulse"></div>
      <span>Loading cost data…</span>
    </div>
  {:else if !summary || summary.total_entries === 0}
    <div class="empty-state">
      <span class="empty-icon">📊</span>
      <h3>No cost data yet</h3>
      <p>Cost attribution data appears after PAUL ledger entries are extracted via <code>base sync</code>.</p>
    </div>
  {:else}
    <!-- Summary cards -->
    <div class="stats-row">
      <div class="stat-card">
        <span class="stat-label">Total Cost</span>
        <span class="stat-value cost">{fmtCost(summary.total_cost)}</span>
        <span class="stat-sub">{summary.total_entries} ledger entries</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">Phases Tracked</span>
        <span class="stat-value">{summary.phases.length}</span>
        <span class="stat-sub">{summary.phases.reduce((a, p) => a + p.session_count, 0)} total sessions</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">Most Expensive</span>
        <span class="stat-value phase-name">{mostExpensive ? mostExpensive.phase : '—'}</span>
        <span class="stat-sub">{mostExpensive ? fmtCost(mostExpensive.total_cost) : ''}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">Avg Cost / Entry</span>
        <span class="stat-value cost">{fmtCost(avgCost)}</span>
        <span class="stat-sub">across all phases</span>
      </div>
    </div>

    <!-- Phase breakdown chart -->
    <div class="section">
      <div class="section-header">
        <h3>Cost by Phase</h3>
        {#if selectedPhase}
          <button class="clear-btn" on:click={() => { selectedPhase = null; }}>✕ Clear selection</button>
        {/if}
      </div>
      <div class="chart-container" bind:this={chartEl}></div>
    </div>

    <!-- Action drill-down -->
    {#if selectedPhase}
      <div class="section drill-down">
        <h3>
          <span class="drill-phase">{selectedPhase.phase}</span>
          <span class="drill-meta">{fmtCost(selectedPhase.total_cost)} · {selectedPhase.session_count} sessions</span>
        </h3>
        <div class="action-table-wrap">
          <table class="action-table">
            <thead>
              <tr>
                <th>Action</th>
                <th class="num">Cost</th>
                <th class="num">Sessions</th>
                <th class="num">Avg / Session</th>
              </tr>
            </thead>
            <tbody>
              {#each selectedPhase.actions.sort((a, b) => b.cost - a.cost) as action}
                <tr>
                  <td>
                    <span class="action-badge" style="background: {actionColor(action.action)}">{action.action}</span>
                  </td>
                  <td class="num">{fmtCost(action.cost)}</td>
                  <td class="num">{action.count}</td>
                  <td class="num">{fmtCostCompact(action.count > 0 ? action.cost / action.count : 0)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

    <!-- Ledger event log -->
    {#if recentLedger.length > 0}
      <div class="section">
        <h3>Recent Ledger Entries</h3>
        <div class="ledger-log">
          {#each recentLedger as entry}
            <div class="ledger-row">
              <span class="ledger-time">{timeAgo(entry.timestamp)}</span>
              <span class="action-badge" style="background: {actionColor(entry.action)}">{entry.action}</span>
              <span class="ledger-phase">{entry.phase || '—'}</span>
              {#if entry.plan}<span class="ledger-plan">Plan {entry.plan}</span>{/if}
              {#if entry.session_cost != null}<span class="ledger-cost">{fmtCostCompact(entry.session_cost)}</span>{/if}
              {#if entry.note}<span class="ledger-note">{entry.note}</span>{/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .cost-panel { padding: 0; }

  /* ─── Loading & Empty ──────────────────────────── */
  .loading {
    display: flex; align-items: center; gap: 12px; justify-content: center;
    padding: 60px 0; color: var(--ink-tertiary);
  }
  .loading-pulse {
    width: 12px; height: 12px; border-radius: 50%;
    background: var(--accent-purple); animation: pulse 1.2s infinite;
  }
  @keyframes pulse { 0%, 100% { opacity: 0.3; } 50% { opacity: 1; } }

  .empty-state {
    text-align: center; padding: 80px 20px; color: var(--ink-tertiary);
  }
  .empty-icon { font-size: 36px; display: block; margin-bottom: 12px; }
  .empty-state h3 { color: var(--ink-muted); margin-bottom: 8px; font-size: 16px; }
  .empty-state p { font-size: 13px; max-width: 380px; margin: 0 auto; line-height: 1.5; }
  .empty-state code {
    background: var(--surface-3); padding: 2px 6px; border-radius: 4px;
    font-family: var(--font-mono); font-size: 12px;
  }

  /* ─── Stats Row ────────────────────────────────── */
  .stats-row {
    display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;
    margin-bottom: 24px;
  }
  .stat-card {
    background: var(--surface-2); border: 1px solid var(--hairline); border-radius: 10px;
    padding: 16px 18px; display: flex; flex-direction: column; min-height: 88px;
  }
  .stat-label {
    font-size: 10px; color: var(--ink-tertiary); margin-bottom: 6px;
    text-transform: uppercase; letter-spacing: 0.6px; font-weight: 500;
  }
  .stat-value {
    font-size: 24px; font-weight: 700; color: var(--ink);
    font-variant-numeric: tabular-nums; line-height: 1.1;
  }
  .stat-value.cost { color: var(--green); }
  .stat-value.phase-name { font-size: 15px; color: var(--accent-purple-glow); }
  .stat-sub { font-size: 10px; color: var(--ink-tertiary); margin-top: 4px; }

  @media (max-width: 800px) {
    .stats-row { grid-template-columns: repeat(2, 1fr); }
  }
  @media (max-width: 500px) {
    .stats-row { grid-template-columns: 1fr; }
  }

  /* ─── Sections ─────────────────────────────────── */
  .section {
    background: var(--surface-2); border: 1px solid var(--hairline); border-radius: 10px;
    padding: 20px; margin-bottom: 16px;
  }
  .section h3 {
    font-size: 13px; font-weight: 600; color: var(--ink-muted);
    text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 16px;
  }
  .section-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 16px;
  }
  .section-header h3 { margin-bottom: 0; }
  .clear-btn {
    background: none; border: 1px solid var(--hairline); border-radius: 6px;
    color: var(--ink-tertiary); font-size: 11px; padding: 4px 10px;
    cursor: pointer; font-family: inherit;
    transition: border-color 0.2s, color 0.2s;
  }
  .clear-btn:hover { border-color: var(--ink-subtle); color: var(--ink-muted); }

  /* ─── Chart ────────────────────────────────────── */
  .chart-container { width: 100%; min-height: 60px; }

  /* ─── Drill-down ───────────────────────────────── */
  .drill-down h3 {
    display: flex; align-items: baseline; gap: 10px; margin-bottom: 16px;
  }
  .drill-phase {
    color: var(--accent-purple-glow); font-size: 14px; font-weight: 600;
    text-transform: none; letter-spacing: 0;
  }
  .drill-meta {
    font-size: 12px; color: var(--ink-tertiary); font-weight: 400;
    text-transform: none; letter-spacing: 0;
  }

  .action-table-wrap { overflow-x: auto; }
  .action-table {
    width: 100%; border-collapse: collapse; font-size: 13px;
  }
  .action-table th {
    text-align: left; padding: 8px 12px; font-size: 10px;
    text-transform: uppercase; letter-spacing: 0.5px;
    color: var(--ink-tertiary); font-weight: 500;
    border-bottom: 1px solid var(--hairline);
  }
  .action-table th.num, .action-table td.num { text-align: right; }
  .action-table td {
    padding: 10px 12px; border-bottom: 1px solid var(--hairline);
    color: var(--ink-muted);
  }
  .action-table tr:last-child td { border-bottom: none; }
  .action-table td.num {
    font-variant-numeric: tabular-nums; font-weight: 500; color: var(--ink);
  }

  .action-badge {
    display: inline-block; padding: 2px 8px; border-radius: 4px;
    font-size: 11px; font-weight: 600; color: #090A0C;
    text-transform: capitalize;
  }

  /* ─── Ledger Log ───────────────────────────────── */
  .ledger-log {
    display: flex; flex-direction: column; gap: 2px;
  }
  .ledger-row {
    display: flex; align-items: center; gap: 10px; padding: 8px 4px;
    border-radius: 6px; transition: background 0.15s;
  }
  .ledger-row:hover { background: var(--surface-3); }
  .ledger-time {
    font-size: 11px; color: var(--ink-tertiary); min-width: 60px;
    font-variant-numeric: tabular-nums;
  }
  .ledger-phase {
    font-size: 12px; color: var(--ink-subtle); min-width: 80px;
  }
  .ledger-plan {
    font-size: 11px; color: var(--ink-tertiary);
    background: var(--surface-3); padding: 1px 6px; border-radius: 3px;
    font-family: var(--font-mono);
  }
  .ledger-cost {
    font-size: 11px; color: var(--green); font-weight: 600;
    font-variant-numeric: tabular-nums; margin-left: auto;
  }
  .ledger-note {
    font-size: 11px; color: var(--ink-tertiary); overflow: hidden;
    text-overflow: ellipsis; white-space: nowrap; max-width: 200px;
  }
</style>
