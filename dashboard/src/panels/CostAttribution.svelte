<script>
  import { onMount } from 'svelte';
  import { getLedger, getCostSummary } from '../lib/api.js';

  let ledger = [];
  let loading = true;
  let expandedProject = null;
  let expandedPhase = null;
  let expandedPlan = null;

  function fmtCost(v) {
    if (v == null || v === 0) return '$0.00';
    return '$' + v.toFixed(2);
  }

  function fmtDuration(ms) {
    if (!ms || ms <= 0) return '—';
    const mins = Math.floor(ms / 60000);
    const hrs = Math.floor(mins / 60);
    if (hrs > 0) return `${hrs}h ${mins % 60}m`;
    if (mins > 0) return `${mins}m`;
    return '<1m';
  }

  function fmtDate(ts) {
    if (!ts) return '';
    const d = new Date(ts);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }

  function fmtTime(ts) {
    if (!ts) return '';
    const d = new Date(ts);
    return d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
  }

  const actionColors = {
    plan: { bg: 'var(--primary)', label: 'PLAN' },
    apply: { bg: 'var(--green)', label: 'APPLY' },
    unify: { bg: 'var(--accent-purple)', label: 'UNIFY' },
    iterate: { bg: 'var(--orange)', label: 'ITERATE' },
    discover: { bg: 'var(--accent-cyan)', label: 'DISCOVER' },
    research: { bg: 'var(--accent-lavender)', label: 'RESEARCH' },
  };

  function actionStyle(action) {
    const key = (action || '').toLowerCase();
    return actionColors[key] || { bg: 'var(--ink-tertiary)', label: action };
  }

  // ─── Computed hierarchy: Project → Phase → Plan → Actions ──

  $: hierarchy = buildHierarchy(ledger);

  function buildHierarchy(entries) {
    if (!entries.length) return [];

    const projectMap = new Map();

    for (const e of entries) {
      const proj = e.project || 'Unknown Project';
      if (!projectMap.has(proj)) {
        projectMap.set(proj, { name: proj, phases: new Map(), entries: [] });
      }
      const p = projectMap.get(proj);
      p.entries.push(e);

      const phaseKey = e.phase || 'unknown';
      if (!p.phases.has(phaseKey)) {
        p.phases.set(phaseKey, { phase: phaseKey, plans: new Map(), entries: [] });
      }
      const ph = p.phases.get(phaseKey);
      ph.entries.push(e);

      const planKey = e.plan || 'unknown';
      if (!ph.plans.has(planKey)) {
        ph.plans.set(planKey, { plan: planKey, actions: [] });
      }
      ph.plans.get(planKey).actions.push(e);
    }

    // Compute stats at each level
    const projects = [...projectMap.values()].map(proj => {
      const phases = [...proj.phases.values()].map(ph => {
        const plans = [...ph.plans.values()].map(plan => {
          const sorted = plan.actions.sort((a, b) => (a.timestamp || '').localeCompare(b.timestamp || ''));
          const first = sorted[0]?.timestamp;
          const last = sorted[sorted.length - 1]?.timestamp;
          const duration = first && last ? new Date(last) - new Date(first) : 0;
          const cost = sorted.reduce((s, e) => s + (e.session_cost || 0), 0);
          return { ...plan, duration, cost, first, last, actions: sorted };
        }).sort((a, b) => (a.plan || '').localeCompare(b.plan || ''));

        const first = plans[0]?.first;
        const last = plans[plans.length - 1]?.last;
        const duration = first && last ? new Date(last) - new Date(first) : 0;
        const cost = plans.reduce((s, p) => s + p.cost, 0);
        const actionCount = ph.entries.length;
        return { ...ph, plans, duration, cost, first, last, actionCount };
      }).sort((a, b) => {
        const an = parseInt(a.phase) || 0;
        const bn = parseInt(b.phase) || 0;
        return an - bn;
      });

      const first = phases[0]?.first;
      const last = phases[phases.length - 1]?.last;
      const duration = first && last ? new Date(last) - new Date(first) : 0;
      const cost = phases.reduce((s, p) => s + p.cost, 0);
      const totalEntries = proj.entries.length;
      return { ...proj, phases, duration, cost, first, last, totalEntries };
    });

    return projects;
  }

  // ─── Expand/collapse ──────────────────────────────────────

  function toggleProject(name) {
    expandedProject = expandedProject === name ? null : name;
    expandedPhase = null;
    expandedPlan = null;
  }

  function togglePhase(key) {
    expandedPhase = expandedPhase === key ? null : key;
    expandedPlan = null;
  }

  function togglePlan(key) {
    expandedPlan = expandedPlan === key ? null : key;
  }

  async function loadData() {
    loading = true;
    ledger = (await getLedger()) || [];
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
  {:else if hierarchy.length === 0}
    <div class="empty-state">
      <span class="empty-icon">📊</span>
      <h3>No cost data yet</h3>
      <p>Cost attribution data appears after PAUL ledger entries are extracted via <code>base sync</code>.</p>
    </div>
  {:else}
    {#each hierarchy as project}
      <!-- Project level -->
      <button class="tree-row project-row" class:expanded={expandedProject === project.name} on:click={() => toggleProject(project.name)}>
        <span class="tree-chevron">{expandedProject === project.name ? '▾' : '▸'}</span>
        <span class="tree-name project-name">{project.name}</span>
        <div class="tree-stats">
          <span class="stat-pill entries">{project.totalEntries} entries</span>
          <span class="stat-pill phases">{project.phases.length} phases</span>
          <span class="stat-pill duration">{fmtDuration(project.duration)}</span>
          <span class="stat-pill cost">{fmtCost(project.cost)}</span>
        </div>
      </button>

      {#if expandedProject === project.name}
        {#each project.phases as phase}
          <!-- Phase level -->
          <button class="tree-row phase-row" class:expanded={expandedPhase === phase.phase} on:click={() => togglePhase(phase.phase)}>
            <span class="tree-chevron">{expandedPhase === phase.phase ? '▾' : '▸'}</span>
            <span class="tree-name phase-name">Phase {phase.phase}</span>
            <div class="tree-stats">
              <span class="stat-pill plans">{phase.plans.length} plan{phase.plans.length !== 1 ? 's' : ''}</span>
              <span class="stat-pill actions">{phase.actionCount} actions</span>
              <span class="stat-pill duration">{fmtDuration(phase.duration)}</span>
              <span class="stat-pill cost">{fmtCost(phase.cost)}</span>
            </div>
          </button>

          {#if expandedPhase === phase.phase}
            {#each phase.plans as plan}
              <!-- Plan level -->
              <button class="tree-row plan-row" class:expanded={expandedPlan === plan.plan} on:click={() => togglePlan(plan.plan)}>
                <span class="tree-chevron">{expandedPlan === plan.plan ? '▾' : '▸'}</span>
                <span class="tree-name plan-name">Plan {plan.plan}</span>
                <div class="tree-stats">
                  <span class="stat-pill actions">{plan.actions.length} actions</span>
                  <span class="stat-pill duration">{fmtDuration(plan.duration)}</span>
                  <span class="stat-pill cost">{fmtCost(plan.cost)}</span>
                </div>
              </button>

              {#if expandedPlan === plan.plan}
                <div class="action-list">
                  {#each plan.actions as entry, i}
                    {@const prevTs = i > 0 ? plan.actions[i-1].timestamp : null}
                    {@const gap = prevTs && entry.timestamp ? new Date(entry.timestamp) - new Date(prevTs) : 0}
                    {#if gap > 0}
                      <div class="action-gap">
                        <span class="gap-line"></span>
                        <span class="gap-label">{fmtDuration(gap)}</span>
                        <span class="gap-line"></span>
                      </div>
                    {/if}
                    <div class="action-row">
                      <span class="action-badge" style="background: {actionStyle(entry.action).bg}">{actionStyle(entry.action).label}</span>
                      <span class="action-time">{fmtDate(entry.timestamp)} {fmtTime(entry.timestamp)}</span>
                      {#if entry.session_cost != null && entry.session_cost > 0}
                        <span class="action-cost">{fmtCost(entry.session_cost)}</span>
                      {/if}
                      {#if entry.note}
                        <span class="action-note">{entry.note}</span>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            {/each}
          {/if}
        {/each}
      {/if}
    {/each}
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

  /* ─── Tree Rows ────────────────────────────────── */
  .tree-row {
    display: flex; align-items: center; gap: 8px; width: 100%;
    padding: 12px 16px; border: none; background: none;
    font-family: inherit; font-size: 13px; color: var(--ink);
    cursor: pointer; text-align: left;
    border-bottom: 1px solid var(--hairline);
    transition: background 0.15s;
  }
  .tree-row:hover { background: var(--surface-2); }
  .tree-row.expanded { background: var(--surface-2); }

  .tree-chevron {
    font-size: 11px; color: var(--ink-tertiary); width: 14px;
    flex-shrink: 0; text-align: center;
  }

  .tree-name { font-weight: 500; white-space: nowrap; }
  .project-name { font-size: 14px; color: var(--ink); }

  .phase-row { padding-left: 32px; }
  .phase-name { color: var(--accent-purple-glow); }

  .plan-row { padding-left: 56px; }
  .plan-name { color: var(--primary); font-family: var(--font-mono); font-size: 12px; }

  .tree-stats {
    display: flex; gap: 8px; margin-left: auto; align-items: center;
    flex-shrink: 0;
  }

  .stat-pill {
    font-size: 11px; padding: 2px 8px; border-radius: 4px;
    font-variant-numeric: tabular-nums; white-space: nowrap;
  }
  .stat-pill.entries { color: var(--ink-tertiary); }
  .stat-pill.phases { color: var(--ink-tertiary); }
  .stat-pill.plans { color: var(--ink-tertiary); }
  .stat-pill.actions { color: var(--ink-tertiary); }
  .stat-pill.duration {
    color: var(--accent-cyan); background: rgba(149, 239, 255, 0.08);
  }
  .stat-pill.cost {
    color: var(--green); background: rgba(0, 202, 83, 0.08);
    font-weight: 600;
  }

  /* ─── Action List (leaf level) ─────────────────── */
  .action-list {
    padding: 8px 16px 8px 80px;
    border-bottom: 1px solid var(--hairline);
    background: var(--surface-1);
  }

  .action-row {
    display: flex; align-items: center; gap: 10px;
    padding: 6px 0;
  }

  .action-badge {
    display: inline-block; padding: 2px 8px; border-radius: 4px;
    font-size: 10px; font-weight: 700; color: #090A0C;
    letter-spacing: 0.5px; min-width: 48px; text-align: center;
  }

  .action-time {
    font-size: 12px; color: var(--ink-subtle);
    font-variant-numeric: tabular-nums; min-width: 110px;
  }

  .action-cost {
    font-size: 12px; color: var(--green); font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .action-note {
    font-size: 11px; color: var(--ink-tertiary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
    max-width: 400px;
  }

  /* ─── Duration Gap Indicators ──────────────────── */
  .action-gap {
    display: flex; align-items: center; gap: 8px;
    padding: 2px 0;
  }
  .gap-line {
    flex: 1; height: 1px;
    background: linear-gradient(90deg, transparent, var(--hairline), transparent);
  }
  .gap-label {
    font-size: 10px; color: var(--accent-cyan); font-weight: 500;
    white-space: nowrap;
  }
</style>
