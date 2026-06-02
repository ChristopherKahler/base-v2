<script>
  import GraphExplorer from './panels/GraphExplorer.svelte';
  import OperationsPanel from './panels/OperationsPanel.svelte';
  import SessionActivity from './panels/SessionActivity.svelte';
  import UsageAnalytics from './panels/UsageAnalytics.svelte';

  let activePanel = 'graph';

  const panels = [
    { id: 'graph', label: 'Graph Explorer', color: 'var(--primary)', ready: true },
    { id: 'operations', label: 'Operations', color: 'var(--green)', ready: true },
    { id: 'session', label: 'Session Activity', color: 'var(--accent-cyan)', ready: true },
    { id: 'usage', label: 'Usage Analytics', color: 'var(--accent-purple)', ready: true },
  ];
</script>

<div class="layout">
  <aside class="sidebar">
    <div class="sidebar-brand">
      <h1>BASE</h1>
      <span>Command Center</span>
    </div>
    <nav class="sidebar-nav">
      {#each panels as panel}
        <button
          class="nav-item"
          class:active={activePanel === panel.id}
          class:disabled={!panel.ready}
          on:click={() => panel.ready && (activePanel = panel.id)}
        >
          <span class="dot" style="background: {panel.color}"></span>
          {panel.label}
          {#if !panel.ready}
            <span style="font-size: 10px; color: var(--ink-tertiary); margin-left: auto;">Soon</span>
          {/if}
        </button>
      {/each}
    </nav>
  </aside>

  <main class="main">
    {#if activePanel === 'graph'}
      <GraphExplorer />
    {:else if activePanel === 'operations'}
      <OperationsPanel />
    {:else if activePanel === 'session'}
      <SessionActivity />
    {:else if activePanel === 'usage'}
      <UsageAnalytics />
    {/if}
  </main>
</div>
