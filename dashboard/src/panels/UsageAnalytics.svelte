<script>
  import { onMount, onDestroy } from 'svelte';
  import { getUsageSummary, getUsageSessions, getUsageProjects, exportUsageCsv } from '../lib/api.js';
  import * as d3 from 'd3';

  let summary = null;
  let sessions = [];
  let projects = [];
  let loading = true;
  let chartEl;
  let modelBarEl;
  let trendEl;
  let heatmapEl;
  let sortCol = 'started';
  let days = 90; // default to 90 days like TokenBBQ
  let selectedModel = ''; // empty = all models
  let showAllProjects = false;
  let showAllDaily = false;
  let projSortCol = 'cost';
  let projSortDir = -1;
  let refreshInterval;
  let activeModal = null; // 'tokens' | 'cost' | 'costday' | 'active' | 'model' | null

  function openModal(name) { activeModal = name; }
  function closeModal() { activeModal = null; }

  // Compute activity stats for the modal
  $: activityStats = (() => {
    if (!summary || !summary.daily) return null;
    const daily = summary.daily;
    const dateSet = new Set(daily.map(d => d.date));

    // Streaks: compute from date array
    const sorted = [...daily].sort((a, b) => a.date.localeCompare(b.date));
    let longest = 0, current = 0, streak = 0;
    const today = new Date().toISOString().substring(0, 10);
    for (let i = 0; i < sorted.length; i++) {
      if (i === 0) { streak = 1; }
      else {
        const prev = new Date(sorted[i-1].date);
        const curr = new Date(sorted[i].date);
        const diff = (curr - prev) / 86400000;
        streak = diff <= 1 ? streak + 1 : 1;
      }
      longest = Math.max(longest, streak);
    }
    // Current streak from today backwards
    current = 0;
    const d = new Date();
    for (let i = 0; i < 365; i++) {
      const ds = d.toISOString().substring(0, 10);
      if (dateSet.has(ds)) { current++; d.setDate(d.getDate() - 1); }
      else if (i === 0) { d.setDate(d.getDate() - 1); } // today might not have data yet
      else break;
    }

    // Day of week distribution
    const dow = [0,0,0,0,0,0,0];
    sorted.forEach(d => { const day = new Date(d.date).getDay(); dow[day]++; });
    const dowLabels = ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'];
    const maxDow = Math.max(...dow) || 1;

    // Monthly consistency
    const months = {};
    sorted.forEach(d => { const m = d.date.substring(0, 7); months[m] = (months[m] || 0) + 1; });
    const monthEntries = Object.entries(months).sort((a, b) => a[0].localeCompare(b[0]));
    const maxMonth = Math.max(...monthEntries.map(m => m[1])) || 1;

    // Total tracked days (span from first to last)
    const first = sorted[0]?.date;
    const last = sorted[sorted.length - 1]?.date;
    const totalDays = first && last ? Math.ceil((new Date(last) - new Date(first)) / 86400000) + 1 : 0;
    const coverage = totalDays > 0 ? Math.round((dateSet.size / totalDays) * 100) : 0;
    const avgPerWeek = totalDays > 0 ? (dateSet.size / (totalDays / 7)).toFixed(1) : '0';

    return { longest, current, totalDays, coverage, avgPerWeek, dow, dowLabels, maxDow, monthEntries, maxMonth };
  })();

  async function loadData() {
    const wasLoading = !summary; // first load vs refresh
    if (wasLoading) loading = true;
    const [s, sess, proj] = await Promise.all([
      getUsageSummary(days), getUsageSessions(days), getUsageProjects(days),
    ]);
    summary = s;
    sessions = sess;
    projects = proj;
    if (wasLoading) {
      showAllProjects = false;
      showAllDaily = false;
    }
    loading = false;
    setTimeout(() => {
      if (summary && summary.daily) drawChart(summary.daily);
      if (summary && summary.models) drawModelBars(summary.models);
      if (summary && summary.daily) drawTrend(summary.daily);
      if (summary && summary.daily) drawHeatmap(summary.daily);
    }, 50);
  }

  async function setDays(d) {
    days = d;
    await loadData();
  }

  function toggleModel(name) {
    selectedModel = selectedModel === name ? '' : name;
  }

  // Filter sessions by selected model
  $: filteredSessions = selectedModel
    ? sessions.filter(s => s.model === selectedModel)
    : sessions;

  // Recompute stats when model filter changes
  $: filteredStats = (() => {
    if (!selectedModel || !summary) return null;
    const m = summary.models[selectedModel];
    if (!m) return null;
    return {
      input: m.input,
      output: m.output,
      cost: m.cost,
      count: m.count,
      cacheRead: m.cache_read,
      cacheWrite: m.cache_write,
    };
  })();

  let sortDir = -1;

  function fmt(n) {
    if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(1) + 'B';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return n.toString();
  }

  // Full comma-separated numbers for tables
  function fmtFull(n) {
    return n.toLocaleString();
  }

  function fmtCost(n) {
    return '$' + n.toFixed(2);
  }

  function fmtDate(iso) {
    if (!iso) return '—';
    return iso.substring(0, 10);
  }

  function relTime(iso) {
    if (!iso) return '—';
    const diff = (Date.now() - new Date(iso).getTime()) / 1000;
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function sortProjects(col) {
    if (projSortCol === col) projSortDir *= -1;
    else { projSortCol = col; projSortDir = -1; }
  }

  $: sortedProjects = [...projects].sort((a, b) => {
    const av = a[projSortCol] ?? '';
    const bv = b[projSortCol] ?? '';
    if (typeof av === 'number') return (av - bv) * projSortDir;
    return String(av).localeCompare(String(bv)) * projSortDir;
  });

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

  $: sortedSessions = [...filteredSessions].sort((a, b) => {
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
    const containerH = Math.max(el.clientHeight, 180);
    const height = containerH - margin.top - margin.bottom;

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
      .domain([0, maxVal * 1.05])
      .range([height, 0])
      .nice();

    // Horizontal grid lines
    const yTicks = y.ticks(5);
    svg.selectAll('.h-grid')
      .data(yTicks).enter()
      .append('line')
      .attr('x1', 0).attr('x2', width)
      .attr('y1', d => y(d)).attr('y2', d => y(d))
      .attr('stroke', '#1e1f22').attr('stroke-width', 1);

    // Bars — stacked: input (bottom) + output (top)
    svg.selectAll('.bar-input')
      .data(data).enter()
      .append('rect')
      .attr('x', d => x(d.date))
      .attr('y', d => y(d.input + d.output))
      .attr('width', x.bandwidth())
      .attr('height', d => height - y(d.input))
      .attr('fill', '#E4934A')
      .attr('rx', 2);

    svg.selectAll('.bar-output')
      .data(data).enter()
      .append('rect')
      .attr('x', d => x(d.date))
      .attr('y', d => y(d.output))
      .attr('width', x.bandwidth())
      .attr('height', d => height - y(d.output))
      .attr('fill', '#3FB950')
      .attr('rx', 2);

    // Tooltips — invisible rect overlay per bar, fixed positioning
    const tooltip = d3.select('body').append('div').attr('class', 'chart-tooltip').style('opacity', 0);
    svg.selectAll('.bar-hover')
      .data(data).enter()
      .append('rect')
      .attr('x', d => x(d.date))
      .attr('y', 0)
      .attr('width', x.bandwidth())
      .attr('height', height)
      .attr('fill', 'transparent')
      .on('mouseenter', (e, d) => {
        tooltip.html(`<strong>${d.date}</strong><br>Input: ${fmtFull(d.input)}<br>Output: ${fmtFull(d.output)}<br>Cost: ${fmtCost(d.cost)}`)
          .style('opacity', 1);
      })
      .on('mousemove', (e) => {
        tooltip.style('left', (e.pageX + 12) + 'px').style('top', (e.pageY - 10) + 'px');
      })
      .on('mouseleave', () => tooltip.style('opacity', 0));

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

  function drawModelBars(models) {
    if (!modelBarEl || !models) return;
    const el = modelBarEl;
    el.innerHTML = '';

    const entries = Object.entries(models).sort((a, b) => (b[1].input + b[1].output) - (a[1].input + a[1].output));
    if (entries.length === 0) return;

    const margin = { top: 6, right: 16, bottom: 30, left: 140 };
    const barHeight = 28;
    const gap = 8;
    const height = entries.length * (barHeight + gap) + margin.top + margin.bottom;
    const width = el.clientWidth - margin.left - margin.right;

    const svg = d3.select(el)
      .append('svg')
      .attr('width', width + margin.left + margin.right)
      .attr('height', height)
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const maxTokens = d3.max(entries, d => d[1].input + d[1].output) || 1;
    const x = d3.scaleLinear().domain([0, maxTokens * 1.05]).range([0, width]).nice();

    // Grid lines
    const ticks = x.ticks(5);
    svg.selectAll('.grid-line')
      .data(ticks).enter()
      .append('line')
      .attr('x1', d => x(d)).attr('x2', d => x(d))
      .attr('y1', 0).attr('y2', entries.length * (barHeight + gap))
      .attr('stroke', '#1e1f22').attr('stroke-width', 1);

    // X-axis
    svg.append('g')
      .attr('transform', `translate(0,${entries.length * (barHeight + gap)})`)
      .call(d3.axisBottom(x).ticks(5).tickFormat(d => fmt(d)))
      .selectAll('text').attr('fill', '#68686A').style('font-size', '9px');
    svg.selectAll('.domain').attr('stroke', '#2D2F31');
    svg.selectAll('.tick line').attr('stroke', '#2D2F31');

    // Tooltip
    const tooltip = d3.select('body').append('div').attr('class', 'chart-tooltip').style('opacity', 0);

    entries.forEach(([name, data], i) => {
      const y = i * (barHeight + gap);
      const total = data.input + data.output;

      // Bar with gradient effect
      const defs = svg.append('defs');
      const gradId = `bar-grad-${i}`;
      const grad = defs.append('linearGradient').attr('id', gradId);
      grad.append('stop').attr('offset', '0%').attr('stop-color', '#E4934A');
      grad.append('stop').attr('offset', '100%').attr('stop-color', '#c47a3a');

      svg.append('rect')
        .attr('x', 0).attr('y', y)
        .attr('width', Math.max(x(total), 2)).attr('height', barHeight)
        .attr('fill', `url(#${gradId})`).attr('rx', 3)
        .on('mouseenter', (e) => {
          tooltip.html(`<div style="display:flex;align-items:center;gap:8px;margin-bottom:4px"><span style="display:inline-block;width:10px;height:10px;background:#E4934A;border-radius:2px"></span><strong>${name.replace('claude-','')}</strong></div><div style="font-variant-numeric:tabular-nums">${fmtFull(total)} tokens · ${fmtCost(data.cost)}</div>`)
            .style('opacity', 1);
        })
        .on('mousemove', (e) => {
          tooltip.style('left', (e.pageX + 14) + 'px').style('top', (e.pageY - 12) + 'px');
        })
        .on('mouseleave', () => tooltip.style('opacity', 0));

      // Label (left)
      svg.append('text')
        .attr('x', -10).attr('y', y + barHeight / 2)
        .attr('text-anchor', 'end').attr('dominant-baseline', 'middle')
        .attr('fill', '#B0B0B2').style('font-size', '12px').style('font-weight', '500')
        .text(name.replace('claude-', '') + ' · Claude Code');
    });
  }

  function drawTrend(data) {
    if (!trendEl || !data || data.length < 1) return;
    const el = trendEl;
    el.innerHTML = '';

    // Aggregate by month
    const monthMap = {};
    data.forEach(d => {
      const month = d.date.substring(0, 7);
      monthMap[month] = (monthMap[month] || 0) + d.cost;
    });
    let months = Object.entries(monthMap).sort((a, b) => a[0].localeCompare(b[0]))
      .map(([month, cost]) => ({ month, cost }));

    if (months.length < 1) return;

    // Always pad with zero-start point for a nice curve from origin
    const firstMonth = months[0].month;
    const fmNum = parseInt(firstMonth.substring(5));
    const fmYear = parseInt(firstMonth.substring(0, 4));
    const prevMonth = fmNum === 1
      ? `${fmYear - 1}-12`
      : `${fmYear}-${String(fmNum - 1).padStart(2, '0')}`;
    months = [{ month: prevMonth, cost: 0 }, ...months];

    const margin = { top: 12, right: 16, bottom: 32, left: 56 };
    const width = el.clientWidth - margin.left - margin.right;
    const height = 260 - margin.top - margin.bottom;

    const svg = d3.select(el)
      .append('svg')
      .attr('width', width + margin.left + margin.right)
      .attr('height', height + margin.top + margin.bottom)
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scalePoint().domain(months.map(d => d.month)).range([0, width]).padding(0.3);
    const yMax = d3.max(months, d => d.cost) || 1;
    const y = d3.scaleLinear().domain([0, yMax * 1.1]).range([height, 0]).nice();

    // Horizontal grid lines
    const yTicks = y.ticks(6);
    svg.selectAll('.h-grid')
      .data(yTicks).enter()
      .append('line')
      .attr('x1', 0).attr('x2', width)
      .attr('y1', d => y(d)).attr('y2', d => y(d))
      .attr('stroke', '#1e1f22').attr('stroke-width', 1);

    // Area fill
    const area = d3.area()
      .x(d => x(d.month))
      .y0(height)
      .y1(d => y(d.cost))
      .curve(d3.curveMonotoneX);

    svg.append('path')
      .datum(months)
      .attr('d', area)
      .attr('fill', 'rgba(228, 147, 74, 0.15)');

    // Line
    const line = d3.line()
      .x(d => x(d.month))
      .y(d => y(d.cost))
      .curve(d3.curveMonotoneX);

    svg.append('path')
      .datum(months)
      .attr('d', line)
      .attr('fill', 'none')
      .attr('stroke', '#E4934A')
      .attr('stroke-width', 2.5);

    // Tooltip
    const tooltip = d3.select('body').append('div').attr('class', 'chart-tooltip').style('opacity', 0);

    // Dots + hover
    svg.selectAll('.dot')
      .data(months).enter()
      .append('circle')
      .attr('cx', d => x(d.month))
      .attr('cy', d => y(d.cost))
      .attr('r', 5)
      .attr('fill', '#E4934A')
      .attr('stroke', '#090A0C')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .on('mouseenter', (e, d) => {
        tooltip.html(`<div style="display:flex;align-items:center;gap:8px;margin-bottom:3px"><strong>${d.month}</strong></div><div style="display:flex;align-items:center;gap:6px"><span style="display:inline-block;width:10px;height:10px;background:#E4934A;border-radius:2px"></span>${fmtCost(d.cost)}</div>`)
          .style('opacity', 1);
        d3.select(e.target).attr('r', 7);
      })
      .on('mousemove', (e) => {
        tooltip.style('left', (e.pageX + 14) + 'px').style('top', (e.pageY - 14) + 'px');
      })
      .on('mouseleave', (e) => {
        tooltip.style('opacity', 0);
        d3.select(e.target).attr('r', 5);
      });

    // X-axis
    svg.append('g')
      .attr('transform', `translate(0,${height})`)
      .call(d3.axisBottom(x))
      .selectAll('text').attr('fill', '#68686A').style('font-size', '10px');

    // Y-axis
    svg.append('g')
      .call(d3.axisLeft(y).ticks(6).tickFormat(d => '$' + fmt(d)))
      .selectAll('text').attr('fill', '#68686A').style('font-size', '10px');

    svg.selectAll('.domain').attr('stroke', '#2D2F31');
    svg.selectAll('.tick line').attr('stroke', '#2D2F31');
  }

  function drawHeatmap(data) {
    if (!heatmapEl || !data) return;
    const el = heatmapEl;
    el.innerHTML = '';

    // Build a map of date -> cost for coloring
    const costMap = {};
    data.forEach(d => { costMap[d.date] = d.cost; });

    // Generate last 90 days of dates
    const today = new Date();
    const dates = [];
    for (let i = 89; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const iso = d.toISOString().substring(0, 10);
      dates.push({ date: iso, cost: costMap[iso] || 0 });
    }

    const cellSize = 16;
    const cellGap = 3;
    const cols = Math.ceil(dates.length / 7);
    const width = cols * (cellSize + cellGap) + 4;
    const height = 7 * (cellSize + cellGap) + 4;

    const maxCost = d3.max(dates, d => d.cost) || 1;
    const colorScale = d3.scaleLinear()
      .domain([0, maxCost * 0.25, maxCost * 0.5, maxCost])
      .range(['#1a1a1c', '#1a4d2e', '#2d8a4e', '#3fb950']);

    const svg = d3.select(el)
      .append('svg')
      .attr('width', width)
      .attr('height', height);

    dates.forEach((d, i) => {
      const col = Math.floor(i / 7);
      const row = i % 7;
      svg.append('rect')
        .attr('x', col * (cellSize + cellGap) + 2)
        .attr('y', row * (cellSize + cellGap) + 2)
        .attr('width', cellSize)
        .attr('height', cellSize)
        .attr('rx', 2)
        .attr('fill', d.cost > 0 ? colorScale(d.cost) : '#1a1a1c')
        .append('title')
        .text(`${d.date}: $${d.cost.toFixed(2)}`);
    });
  }

  onMount(() => {
    loadData();
    // Auto-refresh every 30 seconds for live number updates
    refreshInterval = setInterval(loadData, 30_000);
  });

  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
  });
</script>

<div class="main-header">
  <h2>Usage Analytics</h2>
  <div style="display: flex; align-items: center; gap: 6px;">
    {#each [7, 30, 90] as d}
      <button class="graph-btn" class:active={days === d} on:click={() => setDays(d)}>{d}d</button>
    {/each}
    <button class="graph-btn" on:click={() => exportUsageCsv(days)}>↓ CSV</button>
  </div>
  {#if summary}
    <span class="header-meta">
      {#if selectedModel}<button class="clear-filter" on:click={() => selectedModel = ''}>✕ {selectedModel.replace('claude-', '')}</button> · {/if}
      Last {days}d · {filteredSessions.length} sessions
    </span>
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
    <!-- Stats cards (clickable) -->
    <div class="stats-row">
      <button class="stat-card clickable" on:click={() => openModal('tokens')}>
        <span class="stat-label">Total Tokens</span>
        <span class="stat-value">{fmt(summary.total_input + summary.total_output)}</span>
        <span class="stat-sub">{fmt(summary.total_input)} in · {fmt(summary.total_output)} out</span>
      </button>
      <button class="stat-card clickable" on:click={() => openModal('cost')}>
        <span class="stat-label">Total Cost</span>
        <span class="stat-value cost">{fmtCost(summary.estimated_cost_usd)}</span>
        <span class="stat-sub">cache: {fmt(summary.total_cache_read)} read · {fmt(summary.total_cache_write)} write</span>
      </button>
      <button class="stat-card clickable" on:click={() => openModal('costday')}>
        <span class="stat-label">Cost / Day</span>
        <span class="stat-value cost">{fmtCost(summary.cost_per_day || 0)}</span>
        <span class="stat-sub">avg over {summary.active_days || 0} days</span>
      </button>
      <button class="stat-card clickable" on:click={() => openModal('active')}>
        <span class="stat-label">Active Days</span>
        <span class="stat-value">{summary.active_days || 0}</span>
        <span class="stat-sub">last {days} days</span>
      </button>
      <button class="stat-card clickable" on:click={() => openModal('model')}>
        <span class="stat-label">Top Model</span>
        <span class="stat-value model">{(summary.top_model || primaryModel(summary.models)).replace('claude-', '').replace(/-\d+$/, '')}</span>
        <span class="stat-sub">{Object.keys(summary.models).length} model{Object.keys(summary.models).length !== 1 ? 's' : ''} used</span>
      </button>
    </div>

    <!-- Modal overlay -->
    {#if activeModal}
    <div class="modal-overlay" on:click={closeModal}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={closeModal}>✕</button>

        {#if activeModal === 'active' && activityStats}
        <h3>Activity Analysis</h3>
        <div class="modal-stats-grid">
          <div class="modal-stat-ring">
            <svg viewBox="0 0 80 80" width="80" height="80">
              <circle cx="40" cy="40" r="34" fill="none" stroke="#2D2F31" stroke-width="6"/>
              <circle cx="40" cy="40" r="34" fill="none" stroke="#3FB950" stroke-width="6"
                stroke-dasharray="{2 * Math.PI * 34}" stroke-dashoffset="{2 * Math.PI * 34 * (1 - activityStats.coverage / 100)}"
                transform="rotate(-90 40 40)" stroke-linecap="round"/>
              <text x="40" y="44" text-anchor="middle" fill="#fff" font-size="14" font-weight="700">{activityStats.coverage}%</text>
            </svg>
            <span class="ring-label">Coverage</span>
          </div>
          <div class="modal-stat-card"><span class="modal-stat-val">{activityStats.longest} d</span><span class="modal-stat-lbl">LONGEST STREAK</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{activityStats.current} d</span><span class="modal-stat-lbl">CURRENT STREAK</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{activityStats.avgPerWeek}</span><span class="modal-stat-lbl">AVG DAYS / WEEK</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{activityStats.totalDays} d</span><span class="modal-stat-lbl">TOTAL TRACKED</span></div>
        </div>
        <h4>ACTIVITY BY DAY OF WEEK</h4>
        <div class="dow-bars">
          {#each activityStats.dowLabels as label, i}
          <div class="dow-row">
            <span class="dow-label">{label}</span>
            <div class="dow-bar-track"><div class="dow-bar-fill" style="width: {(activityStats.dow[i] / activityStats.maxDow) * 100}%"></div></div>
            <span class="dow-count">{activityStats.dow[i]} d</span>
          </div>
          {/each}
        </div>
        <h4>MONTHLY CONSISTENCY</h4>
        <div class="dow-bars">
          {#each activityStats.monthEntries as [month, count]}
          <div class="dow-row">
            <span class="dow-label month-label">{month}</span>
            <div class="dow-bar-track"><div class="dow-bar-fill monthly" style="width: {(count / activityStats.maxMonth) * 100}%"></div></div>
            <span class="dow-count">{count} d</span>
          </div>
          {/each}
        </div>

        {:else if activeModal === 'tokens'}
        <h3>Token Breakdown</h3>
        <div class="modal-stats-grid flat">
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(summary.total_input)}</span><span class="modal-stat-lbl">INPUT TOKENS</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(summary.total_output)}</span><span class="modal-stat-lbl">OUTPUT TOKENS</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(summary.total_cache_read)}</span><span class="modal-stat-lbl">CACHE READ</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(summary.total_cache_write)}</span><span class="modal-stat-lbl">CACHE WRITE</span></div>
        </div>
        <h4>BY MODEL</h4>
        {#each Object.entries(summary.models).sort((a, b) => (b[1].input + b[1].output) - (a[1].input + a[1].output)) as [name, data]}
        <div class="modal-model-row">
          <span class="model-dot" style="background: {modelColor(name)}"></span>
          <span class="modal-model-name">{name}</span>
          <span class="modal-model-val">{fmtFull(data.input + data.output)} tokens</span>
          <span class="modal-model-cost">{fmtCost(data.cost)}</span>
        </div>
        {/each}

        {:else if activeModal === 'cost'}
        <h3>Cost Analysis</h3>
        <div class="modal-stats-grid flat">
          <div class="modal-stat-card"><span class="modal-stat-val cost">{fmtCost(summary.estimated_cost_usd)}</span><span class="modal-stat-lbl">TOTAL COST</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtCost(summary.cost_per_day || 0)}</span><span class="modal-stat-lbl">AVG / DAY</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtCost((summary.estimated_cost_usd / (summary.active_days / 7 || 1)))}</span><span class="modal-stat-lbl">AVG / WEEK</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{summary.session_count}</span><span class="modal-stat-lbl">SESSIONS</span></div>
        </div>
        <h4>TOP 5 COSTLIEST DAYS</h4>
        {#each [...summary.daily].sort((a, b) => b.cost - a.cost).slice(0, 5) as d, i}
        <div class="modal-model-row">
          <span class="modal-rank">#{i + 1}</span>
          <span class="modal-model-name">{d.date}</span>
          <span class="modal-model-val">{fmtFull(d.input + d.output)} tokens</span>
          <span class="modal-model-cost">{fmtCost(d.cost)}</span>
        </div>
        {/each}

        {:else if activeModal === 'costday'}
        <h3>Daily Cost Trend</h3>
        <div class="modal-stats-grid flat">
          <div class="modal-stat-card"><span class="modal-stat-val cost">{fmtCost(summary.cost_per_day || 0)}</span><span class="modal-stat-lbl">AVERAGE</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtCost(Math.max(...summary.daily.map(d => d.cost)))}</span><span class="modal-stat-lbl">PEAK DAY</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{fmtCost(Math.min(...summary.daily.map(d => d.cost)))}</span><span class="modal-stat-lbl">LOWEST DAY</span></div>
          <div class="modal-stat-card"><span class="modal-stat-val">{summary.active_days}</span><span class="modal-stat-lbl">ACTIVE DAYS</span></div>
        </div>

        {:else if activeModal === 'model'}
        <h3>Model Usage</h3>
        {#each Object.entries(summary.models).sort((a, b) => (b[1].input + b[1].output) - (a[1].input + a[1].output)) as [name, data]}
        <div class="modal-model-detail">
          <div class="modal-model-header">
            <span class="model-dot" style="background: {modelColor(name)}"></span>
            <span class="modal-model-name">{name}</span>
          </div>
          <div class="modal-stats-grid flat compact">
            <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(data.input)}</span><span class="modal-stat-lbl">INPUT</span></div>
            <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(data.output)}</span><span class="modal-stat-lbl">OUTPUT</span></div>
            <div class="modal-stat-card"><span class="modal-stat-val">{fmtFull(data.cache_read)}</span><span class="modal-stat-lbl">CACHE R</span></div>
            <div class="modal-stat-card"><span class="modal-stat-val cost">{fmtCost(data.cost)}</span><span class="modal-stat-lbl">COST</span></div>
          </div>
        </div>
        {/each}
        {/if}
      </div>
    </div>
    {/if}

    <!-- Charts 2×2 grid -->
    <div class="charts-grid">
      <div class="chart-panel">
        <h3>Daily Token Usage</h3>
        <div class="legend">
          <span class="legend-item"><span class="legend-dot" style="background: #E4934A"></span>Input</span>
          <span class="legend-item"><span class="legend-dot" style="background: #3FB950"></span>Output</span>
        </div>
        <div class="chart-container" bind:this={chartEl}></div>
      </div>
      <div class="chart-panel">
        <h3>Top Models by Tokens</h3>
        <div class="chart-container model-bar-container" bind:this={modelBarEl}></div>
      </div>
      <div class="chart-panel">
        <h3>Monthly Trend</h3>
        <div class="chart-container" bind:this={trendEl}></div>
      </div>
      <div class="chart-panel">
        <h3>Activity <span class="activity-sub">(Last 90 Days)</span></h3>
        <div class="heatmap-container" bind:this={heatmapEl}></div>
        <div class="model-list compact-models">
          {#each Object.entries(summary.models).sort((a, b) => (b[1].input + b[1].output) - (a[1].input + a[1].output)).slice(0, 4) as [name, data]}
            <div class="model-row">
              <span class="model-dot" style="background: {modelColor(name)}"></span>
              <span class="model-name">{name.replace('claude-', '')}</span>
              <span class="model-tokens">{fmt(data.input + data.output)}</span>
              <span class="model-cost">{fmtCost(data.cost)}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Projects table -->
    {#if projects && projects.length > 0}
    <div class="projects-section">
      <h3>Projects</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Project</th>
              <th>Providers</th>
              {#each [['total_tokens', 'Tokens'], ['cost', 'Cost'], ['event_count', 'Events'], ['last_active', 'Last Active']] as [col, label]}
                <th class="num sortable" on:click={() => sortProjects(col)} class:sorted={projSortCol === col}>
                  {label}{#if projSortCol === col}<span class="sort-arrow">{projSortDir > 0 ? '↑' : '↓'}</span>{/if}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each (showAllProjects ? sortedProjects : sortedProjects.slice(0, 10)) as p}
              <tr>
                <td class="project-cell">· {p.project}</td>
                <td><span class="provider-badge">{p.provider}</span></td>
                <td class="num">{fmtFull(p.total_tokens)}</td>
                <td class="num cost-cell">{fmtCost(p.cost)}</td>
                <td class="num">{fmtFull(p.event_count)}</td>
                <td class="num date-cell">{fmtDate(p.last_active)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if projects.length > 10}
          <button class="show-all-btn" on:click={() => showAllProjects = !showAllProjects}>
            {showAllProjects ? 'Show less' : `Show all (${projects.length})`}
          </button>
        {/if}
      </div>
    </div>
    {/if}

    <!-- Daily Breakdown table -->
    {#if summary && summary.daily && summary.daily.length > 0}
    <div class="daily-section">
      <h3>Daily Breakdown</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Date</th>
              <th class="num">Sources</th>
              <th class="num">Input</th>
              <th class="num">Output</th>
              <th class="num">Cache R</th>
              <th class="num">Cache W</th>
              <th class="num">Cost</th>
            </tr>
          </thead>
          <tbody>
            {#each (showAllDaily ? [...summary.daily].reverse() : [...summary.daily].reverse().slice(0, 14)) as d}
              <tr>
                <td class="date-cell">· {d.date}</td>
                <td class="num">{fmtFull(d.sources || 0)}</td>
                <td class="num">{fmtFull(d.input)}</td>
                <td class="num">{fmtFull(d.output)}</td>
                <td class="num">{fmtFull(d.cache_read || 0)}</td>
                <td class="num">{fmtFull(d.cache_write || 0)}</td>
                <td class="num cost-cell">{fmtCost(d.cost)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if summary.daily.length > 14}
          <button class="show-all-btn" on:click={() => showAllDaily = !showAllDaily}>
            {showAllDaily ? 'Show less' : `Show all (${summary.daily.length})`}
          </button>
        {/if}
      </div>
    </div>
    {/if}

    <!-- Session table with inline model filter -->
    <div class="sessions-section">
      <div class="sessions-header">
        <h3>Recent Sessions</h3>
        <div class="filter-pills">
          <button class="filter-pill" class:active={!selectedModel} on:click={() => selectedModel = ''}>All</button>
          {#each Object.entries(summary.models).sort((a, b) => b[1].count - a[1].count) as [name]}
            <button class="filter-pill" class:active={selectedModel === name} on:click={() => toggleModel(name)}>
              <span class="pill-dot" style="background: {modelColor(name)}"></span>{name.replace('claude-', '')}
            </button>
          {/each}
        </div>
      </div>
      <div class="table-wrap">
        <table class="fixed-table">
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
                <td class="num">{fmtFull(s.input)}</td>
                <td class="num">{fmtFull(s.output)}</td>
                <td class="num cost-cell">{fmtCost(s.cost)}</td>
                <td class="num date-cell">{fmtDate(s.started)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

<style>
  /* ═══════════════════════════════════════════════════
     Usage Analytics — Clean Design System
     Inspired by TokenBBQ · Dark canvas · Orange accent
     ═══════════════════════════════════════════════════ */

  /* ─── Layout Shell ─────────────────────────────── */
  .main-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 28px; border-bottom: 1px solid var(--border);
  }
  .main-header h2 { margin: 0; font-size: 15px; font-weight: 600; color: var(--ink-primary); }
  .header-meta { font-size: 11px; color: var(--ink-tertiary); display: flex; align-items: center; gap: 6px; }
  .clear-filter {
    font-size: 10px; padding: 2px 8px; border-radius: 4px;
    background: rgba(228, 147, 74, 0.12); border: 1px solid #E4934A;
    color: #E4934A; cursor: pointer; font-family: inherit;
  }
  .main-content { flex: 1; overflow-y: auto; padding: 20px 28px; }
  .loading, .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    height: 200px; color: var(--ink-tertiary);
  }
  .empty-state h3 { margin: 0 0 6px; font-size: 15px; font-weight: 500; }
  .empty-state p { margin: 0; font-size: 12px; }

  /* ─── Stat Cards ───────────────────────────────── */
  .stats-row {
    display: grid; grid-template-columns: repeat(5, 1fr);
    gap: 10px; margin-bottom: 20px;
  }
  .stat-card {
    background: var(--surface-02); border: 1px solid var(--border); border-radius: 10px;
    padding: 16px 18px; display: flex; flex-direction: column; min-height: 88px;
  }
  .stat-card.clickable {
    cursor: pointer; text-align: left; font-family: inherit; color: inherit;
    transition: border-color 0.2s, background 0.2s, transform 0.15s;
  }
  .stat-card.clickable:hover {
    border-color: #E4934A; background: rgba(228, 147, 74, 0.04);
    transform: translateY(-1px);
  }
  .stat-label {
    font-size: 10px; color: var(--ink-tertiary); margin-bottom: 6px;
    text-transform: uppercase; letter-spacing: 0.6px; font-weight: 500;
  }
  .stat-value {
    font-size: 24px; font-weight: 700; color: var(--ink-primary);
    font-variant-numeric: tabular-nums; line-height: 1.1;
  }
  .stat-value.cost { color: var(--green); }
  .stat-value.model { font-size: 17px; }
  .stat-sub { font-size: 10px; color: var(--ink-tertiary); margin-top: 4px; }

  /* ─── Charts Grid ──────────────────────────────── */
  .charts-grid {
    display: grid; grid-template-columns: 1fr 1fr;
    gap: 10px; margin-bottom: 20px;
    grid-auto-rows: 1fr;
  }
  .chart-panel {
    background: var(--surface-02); border: 1px solid var(--border);
    border-radius: 10px; padding: 16px 18px;
    display: flex; flex-direction: column;
  }
  .chart-panel .chart-container { flex: 1; min-height: 180px; }
  .chart-panel h3 {
    margin: 0 0 10px; font-size: 11px; font-weight: 600;
    color: var(--ink-secondary); text-transform: uppercase; letter-spacing: 0.5px;
  }
  .activity-sub { font-weight: 400; color: var(--ink-tertiary); }
  .model-bar-container { min-height: 160px; height: auto; }
  .heatmap-container { margin-bottom: 12px; overflow-x: auto; display: flex; justify-content: center; }
  .legend { display: flex; gap: 14px; margin-bottom: 8px; }
  .legend-item { font-size: 10px; color: var(--ink-tertiary); display: flex; align-items: center; gap: 5px; }
  .legend-dot { width: 8px; height: 8px; border-radius: 2px; flex-shrink: 0; }
  .chart-container { width: 100%; position: relative; }

  /* ─── Model List (in Activity chart panel) ─────── */
  .model-list { display: flex; flex-direction: column; gap: 2px; }
  .compact-models .model-row { padding: 5px 8px; }
  .model-row {
    display: grid; grid-template-columns: 10px 1fr auto auto;
    align-items: center; gap: 10px; padding: 6px 10px;
    border-radius: 6px; font-family: inherit; color: inherit;
  }
  .model-row:hover { background: var(--surface-03); }
  .model-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .model-name { font-size: 12px; color: var(--ink-primary); font-weight: 600; }
  .model-tokens { font-size: 11px; color: var(--ink-secondary); font-variant-numeric: tabular-nums; text-align: right; }
  .model-cost { font-size: 11px; color: var(--green); font-variant-numeric: tabular-nums; text-align: right; font-weight: 600; }

  /* ─── Tables — Shared Foundation ────────────────── */
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; table-layout: fixed; }
  th {
    padding: 10px 14px; font-size: 10px; font-weight: 600;
    color: var(--ink-tertiary); text-transform: uppercase; letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border); cursor: pointer; user-select: none;
    white-space: nowrap;
  }
  th:hover { color: var(--ink-secondary); }
  th.sorted { color: var(--ink-primary); }
  th.num, th.sortable { text-align: right; }
  .sort-arrow { margin-left: 3px; font-size: 9px; }
  td {
    padding: 10px 14px; font-size: 13px; color: var(--ink-secondary);
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  tr:hover td { background: rgba(255, 255, 255, 0.02); }
  td.num { font-variant-numeric: tabular-nums; text-align: right; }
  td.cost-cell { color: var(--green); font-weight: 600; text-align: right; }
  td.date-cell { font-variant-numeric: tabular-nums; color: var(--ink-tertiary); text-align: right; }
  td.project-cell { font-weight: 500; color: var(--ink-primary); }

  /* ─── Section Headers ──────────────────────────── */
  .projects-section, .daily-section, .sessions-section { margin-bottom: 24px; }
  .projects-section h3, .daily-section h3, .sessions-section h3 {
    margin: 0 0 12px; font-size: 14px; font-weight: 700; color: var(--ink-primary);
  }

  /* ─── Projects Table ───────────────────────────── */
  .projects-section table th:nth-child(1) { width: 30%; text-align: left; }
  .projects-section table th:nth-child(2) { width: 12%; text-align: left; }
  .projects-section table th:nth-child(3) { width: 16%; text-align: right; }
  .projects-section table th:nth-child(4) { width: 12%; text-align: right; }
  .projects-section table th:nth-child(5) { width: 12%; text-align: right; }
  .projects-section table th:nth-child(6) { width: 18%; text-align: right; }
  .provider-badge {
    display: inline-block; font-size: 10px; font-weight: 600; padding: 3px 10px;
    border-radius: 4px; background: rgba(228, 147, 74, 0.12); color: #E4934A;
    letter-spacing: 0.3px;
  }

  /* ─── Daily Breakdown Table ────────────────────── */
  .daily-section table td:nth-child(1) { text-align: left; color: var(--ink-primary); font-weight: 500; }
  .daily-section table th:nth-child(1) { width: 18%; text-align: left; }
  .daily-section table th:nth-child(2) { width: 10%; text-align: right; }
  .daily-section table th:nth-child(3) { width: 16%; text-align: right; }
  .daily-section table th:nth-child(4) { width: 16%; text-align: right; }
  .daily-section table th:nth-child(5) { width: 14%; text-align: right; }
  .daily-section table th:nth-child(6) { width: 14%; text-align: right; }
  .daily-section table th:nth-child(7) { width: 12%; text-align: right; }

  /* ─── Sessions Table ───────────────────────────── */
  .sessions-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .sessions-header h3 { margin: 0; }
  .sessions-section table th:nth-child(1) { width: 24%; text-align: left; }
  .sessions-section table th:nth-child(2) { width: 16%; text-align: left; }
  .sessions-section table th:nth-child(3) { width: 15%; text-align: right; }
  .sessions-section table th:nth-child(4) { width: 15%; text-align: right; }
  .sessions-section table th:nth-child(5) { width: 14%; text-align: right; }
  .sessions-section table th:nth-child(6) { width: 16%; text-align: right; }
  .model-badge { font-size: 12px; font-weight: 500; }

  /* ─── Filter Pills ─────────────────────────────── */
  .filter-pills { display: flex; gap: 6px; flex-wrap: wrap; }
  .filter-pill {
    font-size: 11px; padding: 5px 12px; border-radius: 16px;
    background: var(--surface-02); border: 1px solid var(--border);
    color: var(--ink-tertiary); cursor: pointer; font-family: inherit;
    display: flex; align-items: center; gap: 5px; transition: all 0.15s;
  }
  .filter-pill:hover { border-color: var(--ink-tertiary); color: var(--ink-secondary); }
  .filter-pill.active { background: rgba(228, 147, 74, 0.12); border-color: #E4934A; color: #E4934A; }
  .pill-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }

  /* ─── Show All Button ──────────────────────────── */
  .show-all-btn {
    display: block; margin: 12px auto 0; padding: 6px 18px;
    font-size: 11px; color: var(--ink-tertiary); background: none;
    border: 1px solid var(--border); border-radius: 8px; cursor: pointer;
    font-family: inherit; transition: all 0.15s; letter-spacing: 0.3px;
  }
  .show-all-btn:hover { color: var(--ink-secondary); border-color: var(--ink-tertiary); background: var(--surface-02); }

  /* ─── Chart Tooltip ────────────────────────────── */
  :global(.chart-tooltip) {
    position: absolute; pointer-events: none; z-index: 200;
    background: rgba(12, 12, 14, 0.95); border: 1px solid rgba(255,255,255,0.1);
    border-radius: 8px; padding: 10px 14px; font-size: 12px; color: #fff;
    line-height: 1.6; white-space: nowrap; backdrop-filter: blur(8px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  }

  /* ─── Modal ────────────────────────────────────── */
  .modal-overlay {
    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.65); backdrop-filter: blur(4px);
    z-index: 100; display: flex; align-items: center; justify-content: center;
  }
  .modal-content {
    background: #111113; border: 1px solid var(--border);
    border-radius: 14px; padding: 28px; width: 780px; max-width: 92vw;
    max-height: 82vh; overflow-y: auto; overflow-x: hidden; position: relative;
    box-shadow: 0 16px 48px rgba(0,0,0,0.5);
  }
  .modal-content h3 { margin: 0 0 20px; font-size: 20px; font-weight: 700; color: var(--ink-primary); }
  .modal-content h4 {
    margin: 24px 0 12px; font-size: 10px; font-weight: 600; color: var(--ink-tertiary);
    text-transform: uppercase; letter-spacing: 0.8px;
  }
  .modal-close {
    position: absolute; top: 20px; right: 20px;
    background: none; border: none; color: var(--ink-tertiary); font-size: 20px;
    cursor: pointer; padding: 6px 10px; border-radius: 6px; transition: all 0.15s;
  }
  .modal-close:hover { color: var(--ink-primary); background: var(--surface-03); }

  /* ─── Modal Stats Grid ─────────────────────────── */
  .modal-stats-grid {
    display: grid; grid-template-columns: auto 1fr 1fr; gap: 10px; margin-bottom: 10px;
  }
  .modal-stats-grid.flat { grid-template-columns: repeat(4, 1fr); }
  .modal-stats-grid.compact { gap: 8px; margin-bottom: 14px; }
  .modal-stat-ring { grid-row: span 2; display: flex; flex-direction: column; align-items: center; gap: 6px; }
  .ring-label { font-size: 10px; color: var(--ink-tertiary); }
  .modal-stat-card {
    background: var(--surface-02); border: 1px solid var(--border); border-radius: 10px;
    padding: 14px 16px; display: flex; flex-direction: column;
  }
  .modal-stat-val {
    font-size: 22px; font-weight: 700; color: var(--ink-primary);
    font-variant-numeric: tabular-nums; line-height: 1.1;
    overflow: hidden; text-overflow: ellipsis;
  }
  .modal-stat-val.cost { color: var(--green); }
  .modal-stat-lbl {
    font-size: 9px; color: var(--ink-tertiary); text-transform: uppercase;
    letter-spacing: 0.6px; margin-top: 4px;
  }

  /* ─── Modal Bars (DOW / Monthly) ───────────────── */
  .dow-bars { display: flex; flex-direction: column; gap: 8px; }
  .dow-row { display: grid; grid-template-columns: 50px 1fr 44px; align-items: center; gap: 12px; }
  .dow-label { font-size: 12px; color: var(--ink-secondary); font-weight: 500; }
  .month-label { font-size: 11px; width: 70px; }
  .dow-bar-track { height: 12px; background: var(--surface-02); border-radius: 6px; overflow: hidden; }
  .dow-bar-fill { height: 100%; background: #3FB950; border-radius: 6px; transition: width 0.4s ease-out; }
  .dow-bar-fill.monthly { background: var(--primary); }
  .dow-count { font-size: 11px; color: var(--ink-tertiary); text-align: right; font-variant-numeric: tabular-nums; }

  /* ─── Modal Model Rows ─────────────────────────── */
  .modal-model-row {
    display: grid; grid-template-columns: auto 1fr auto auto; align-items: center; gap: 12px;
    padding: 10px 0; border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  .modal-model-name { font-size: 13px; color: var(--ink-primary); font-weight: 500; }
  .modal-model-val { font-size: 12px; color: var(--ink-secondary); font-variant-numeric: tabular-nums; text-align: right; }
  .modal-model-cost { font-size: 13px; color: var(--green); font-weight: 600; font-variant-numeric: tabular-nums; }
  .modal-rank { font-size: 11px; color: var(--ink-tertiary); font-weight: 600; }
  .modal-model-detail { margin-bottom: 16px; }
  .modal-model-header { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
</style>
