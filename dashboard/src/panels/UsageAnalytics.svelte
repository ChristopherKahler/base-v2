<script>
  import { onMount } from 'svelte';
  import { getUsageSummary, getUsageSessions } from '../lib/api.js';
  import * as d3 from 'd3';

  let summary = null;
  let sessions = [];
  let loading = true;
  let chartEl;
  let sortCol = 'started';
  let sortDir = -1;

  function fmt(n) {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return n.toString();
  }

  function fmtCost(n) {
    return '$' + n.toFixed(2);
  }

  function relTime(iso) {
    if (!iso) return '—';
    const diff = (Date.now() - new Date(iso).getTime()) / 1000;
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function primaryModel(models) {
    if (!models) return '—';
    let best = '', bestCount = 0;
    for (const [name, data] of Object.entries(models)) {
      if (data.count > bestCount) { best = name; bestCount = data.count; }
    }
    return best.replace('claude-', '').replace(/-\d+$/, '') || '—';
  }

  function modelColor(name) {
    if (name.includes('opus')) return 'var(--primary)';
    if (name.includes('haiku')) return 'var(--orange)';
    return 'var(--green)';
  }

  function sortTable(col) {
    if (sortCol === col) sortDir *= -1;
    else { sortCol = col; sortDir = -1; }
  }

  $: sortedSessions = [...sessions].sort((a, b) => {
    const av = a[sortCol] ?? '';
    const bv = b[sortCol] ?? '';
    if (typeof av === 'number') return (av - bv) * sortDir;
    return String(av).localeCompare(String(bv)) * sortDir;
  });

  function drawChart(data) {
    if (!chartEl || !data || data.length === 0) return;

    const el = chartEl;
    el.innerHTML = '';

    const margin = { top: 8, right: 12, bottom: 28, left: 50 };
    const width = el.clientWidth - margin.left - margin.right;
    const height = 180 - margin.top - margin.bottom;

    const svg = d3.select(el)
      .append('svg')
      .attr('width', width + margin.left + margin.right)
      .attr('height', height + margin.top + margin.bottom)
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleBand()
      .domain(data.map(d => d.date))
      .range([0, width])
      .padding(0.3);

    const maxVal = d3.max(data, d => d.input + d.output) || 1;
    const y = d3.scaleLinear()
      .domain([0, maxVal])
      .range([height, 0]);

    // Bars — stacked: input (bottom) + output (top)
    svg.selectAll('.bar-input')
      .data(data).enter()
      .append('rect')
      .attr('x', d => x(d.date))
      .attr('y', d => y(d.input + d.output))
      .attr('width', x.bandwidth())
      .attr('height', d => height - y(d.input))
      .attr('fill', '#588BF8')
      .attr('rx', 2);

    svg.selectAll('.bar-output')
      .data(data).enter()
      .append('rect')
      .attr('x', d => x(d.date))
      .attr('y', d => y(d.output))
      .attr('width', x.bandwidth())
      .attr('height', d => height - y(d.output))
      .attr('fill', '#725EFF')
      .attr('rx', 2);

    // X axis — show every Nth label to avoid crowding
    const step = Math.max(1, Math.floor(data.length / 8));
    svg.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x).tickValues(data.filter((_, i) => i % step === 0).map(d => d.date)))
      .selectAll('text')
      .attr('fill', '#68686A')
      .style('font-size', '9px');

    // Y axis
    svg.append('g')
      .call(d3.axisLeft(y).ticks(4).tickFormat(d => fmt(d)))
      .selectAll('text')
      .attr('fill', '#68686A')
      .style('font-size', '9px');

    // Remove axis lines
    svg.selectAll('.domain').attr('stroke', '#2D2F31');
    svg.selectAll('.tick line').attr('stroke', '#2D2F31');
  }

  onMount(async () => {
    const [s, sess] = await Promise.all([getUsageSummary(), getUsageSessions()]);
    summary = s;
    sessions = sess;
    loading = false;

    // Draw chart after DOM update
    setTimeout(() => {
      if (summary && summary.daily) drawChart(summary.daily);
    }, 50);
  });
</script>

<div class="main-header">
  <h2>Usage Analytics</h2>
  {#if summary}
    <span class="header-meta">Last 30 days · {summary.session_count} sessions</span>
  {/if}
</div>

<div class="main-content">
  {#if loading}
    <div class="loading">Parsing session data...</div>
  {:else if !summary || summary.session_count === 0}
    <div class="empty-state">
      <h3>No usage data found</h3>
      <p>Claude Code session files not found at ~/.claude/projects/</p>
    </div>
  {:else}
    <!-- Stats cards -->
    <div class="stats-row">
      <div class="stat-card">
        <span class="stat-label">Total Tokens</span>
        <span class="stat-value">{fmt(summary.total_input + summary.total_output)}</span>
        <span class="stat-sub">{fmt(summary.total_input)} in · {fmt(summary.total_output)} out</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">Estimated Cost</span>
        <span class="stat-value cost">{fmtCost(summary.estimated_cost_usd)}</span>
        <span class="stat-sub">cache: {fmt(summary.total_cache_read)} read · {fmt(summary.total_cache_write)} write</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">Sessions</span>
        <span class="stat-value">{summary.session_count}</span>
        <span class="stat-sub">last 30 days</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">Primary Model</span>
        <span class="stat-value model">{primaryModel(summary.models)}</span>
        <span class="stat-sub">{Object.keys(summary.models).length} model{Object.keys(summary.models).length !== 1 ? 's' : ''} used</span>
      </div>
    </div>

    <!-- Daily chart + Model breakdown side by side -->
    <div class="charts-row">
      <div class="chart-panel">
        <h3>Daily Usage</h3>
        <div class="legend">
          <span class="legend-item"><span class="legend-dot" style="background: #588BF8"></span>Input</span>
          <span class="legend-item"><span class="legend-dot" style="background: #725EFF"></span>Output</span>
        </div>
        <div class="chart-container" bind:this={chartEl}></div>
      </div>
      <div class="model-panel">
        <h3>Model Breakdown</h3>
        <div class="model-list">
          {#each Object.entries(summary.models).sort((a, b) => b[1].count - a[1].count) as [name, data]}
            <div class="model-row">
              <span class="model-dot" style="background: {modelColor(name)}"></span>
              <span class="model-name">{name.replace('claude-', '')}</span>
              <span class="model-count">{data.count} msgs</span>
              <span class="model-tokens">{fmt(data.input + data.output)}</span>
              <span class="model-cost">{fmtCost(data.cost)}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Session table -->
    <div class="sessions-section">
      <h3>Recent Sessions</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              {#each [['project', 'Project'], ['model', 'Model'], ['input', 'Input'], ['output', 'Output'], ['cost', 'Cost'], ['started', 'When']] as [col, label]}
                <th on:click={() => sortTable(col)} class:sorted={sortCol === col}>
                  {label}
                  {#if sortCol === col}<span class="sort-arrow">{sortDir > 0 ? '↑' : '↓'}</span>{/if}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each sortedSessions as s}
              <tr>
                <td class="project-cell">{s.project}</td>
                <td><span class="model-badge" style="color: {modelColor(s.model)}">{s.model.replace('claude-', '')}</span></td>
                <td class="num">{fmt(s.input)}</td>
                <td class="num">{fmt(s.output)}</td>
                <td class="num cost-cell">{fmtCost(s.cost)}</td>
                <td class="time-cell">{relTime(s.started)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
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
  .main-header h2 { margin: 0; font-size: 15px; font-weight: 600; color: var(--ink-primary); }
  .header-meta { font-size: 11px; color: var(--ink-tertiary); }

  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px;
  }
  .loading, .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    height: 200px; color: var(--ink-tertiary);
  }
  .empty-state h3 { margin: 0 0 4px; font-size: 15px; font-weight: 500; }
  .empty-state p { margin: 0; font-size: 12px; }

  /* Stats cards */
  .stats-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 16px;
  }
  .stat-card {
    background: var(--surface-02);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
  }
  .stat-label { font-size: 11px; color: var(--ink-tertiary); margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px; }
  .stat-value { font-size: 22px; font-weight: 700; color: var(--ink-primary); font-variant-numeric: tabular-nums; }
  .stat-value.cost { color: var(--green); }
  .stat-value.model { font-size: 16px; }
  .stat-sub { font-size: 10px; color: var(--ink-tertiary); margin-top: 2px; }

  /* Charts row */
  .charts-row {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 12px;
    margin-bottom: 16px;
  }
  .chart-panel, .model-panel {
    background: var(--surface-02);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
  }
  .chart-panel h3, .model-panel h3, .sessions-section h3 {
    margin: 0 0 10px;
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .legend {
    display: flex;
    gap: 12px;
    margin-bottom: 8px;
  }
  .legend-item { font-size: 10px; color: var(--ink-tertiary); display: flex; align-items: center; gap: 4px; }
  .legend-dot { width: 8px; height: 8px; border-radius: 2px; }
  .chart-container { width: 100%; height: 180px; }

  /* Model breakdown */
  .model-list { display: flex; flex-direction: column; gap: 6px; }
  .model-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
  }
  .model-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
  .model-name { font-size: 12px; color: var(--ink-primary); flex: 1; font-weight: 500; }
  .model-count { font-size: 10px; color: var(--ink-tertiary); }
  .model-tokens { font-size: 11px; color: var(--ink-secondary); font-variant-numeric: tabular-nums; }
  .model-cost { font-size: 11px; color: var(--green); font-variant-numeric: tabular-nums; }

  /* Session table */
  .sessions-section { margin-top: 4px; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; }
  th {
    text-align: left;
    padding: 6px 10px;
    font-size: 10px;
    font-weight: 600;
    color: var(--ink-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    user-select: none;
  }
  th:hover { color: var(--ink-secondary); }
  th.sorted { color: var(--ink-primary); }
  .sort-arrow { margin-left: 2px; }
  td {
    padding: 5px 10px;
    font-size: 12px;
    color: var(--ink-secondary);
    border-bottom: 1px solid rgba(45, 47, 49, 0.5);
  }
  tr:hover td { background: rgba(255, 255, 255, 0.02); }
  .num { font-variant-numeric: tabular-nums; text-align: right; }
  .cost-cell { color: var(--green); }
  .time-cell { color: var(--ink-tertiary); font-size: 11px; }
  .project-cell { font-weight: 500; color: var(--ink-primary); }
  .model-badge { font-size: 11px; font-weight: 500; }
</style>
