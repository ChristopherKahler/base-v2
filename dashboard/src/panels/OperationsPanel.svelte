<script>
  import { onMount } from 'svelte';
  import { getProjects, getDecisions, getReminders, updateTaskStatus } from '../lib/api.js';

  let projects = [];
  let decisions = [];
  let reminders = [];
  let loading = true;
  let viewMode = 'kanban'; // 'kanban' | 'table'
  let filterProject = '';

  // Flatten all tasks from all projects
  $: allTasks = projects.flatMap(p =>
    p.tasks.map(t => ({ ...t, project: p.name, projectStatus: p.status }))
  );

  $: filteredTasks = filterProject
    ? allTasks.filter(t => t.project === filterProject)
    : allTasks;

  $: kanbanColumns = {
    active: filteredTasks.filter(t => t.status === 'active' || t.status === 'in_progress'),
    blocked: filteredTasks.filter(t => t.status === 'blocked'),
    completed: filteredTasks.filter(t => t.status === 'completed' || t.status === 'done'),
    pending: filteredTasks.filter(t => !['active','in_progress','blocked','completed','done'].includes(t.status)),
  };

  let sortCol = 'name';
  let sortDir = 1;

  function sortTable(col) {
    if (sortCol === col) { sortDir *= -1; }
    else { sortCol = col; sortDir = 1; }
  }

  $: sortedTasks = [...filteredTasks].sort((a, b) => {
    const av = a[sortCol] || '';
    const bv = b[sortCol] || '';
    return av.localeCompare(bv) * sortDir;
  });

  function statusColor(status) {
    const map = {
      active: 'var(--green)', in_progress: 'var(--green)',
      blocked: 'var(--red)', completed: 'var(--ink-tertiary)',
      done: 'var(--ink-tertiary)',
    };
    return map[status] || 'var(--ink-subtle)';
  }

  // Drag-and-drop state
  let dragIri = null;
  let dragOverCol = null;

  const statusMap = {
    active: 'active',
    blocked: 'blocked',
    completed: 'completed',
    pending: 'pending',
  };

  function onDragStart(e, task) {
    dragIri = task.iri;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', task.iri);
    // Use only the card element as the drag ghost, not the parent column
    e.dataTransfer.setDragImage(e.currentTarget, e.offsetX, e.offsetY);
  }

  function onDragOver(e, colKey) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    dragOverCol = colKey;
  }

  function onDragLeave(e, colKey) {
    if (dragOverCol === colKey) dragOverCol = null;
  }

  async function onDrop(e, colKey) {
    e.preventDefault();
    dragOverCol = null;
    const iri = dragIri;
    dragIri = null;
    if (!iri) return;

    const newStatus = statusMap[colKey];
    if (!newStatus) return;

    // Optimistic update: move the task locally
    projects = projects.map(p => ({
      ...p,
      tasks: p.tasks.map(t => t.iri === iri ? { ...t, status: newStatus } : t),
    }));

    // Persist to graph
    const result = await updateTaskStatus(iri, newStatus);
    if (!result) {
      // Revert on failure — reload from server
      const [p] = await Promise.all([getProjects()]);
      projects = p;
    }
  }

  function onDragEnd() {
    dragIri = null;
    dragOverCol = null;
  }

  onMount(async () => {
    const [p, d, r] = await Promise.all([getProjects(), getDecisions(), getReminders()]);
    projects = p;
    decisions = d;
    reminders = r;
    loading = false;
  });
</script>

<div class="main-header">
  <h2>Operations</h2>
  <div style="display: flex; align-items: center; gap: var(--sp-md);">
    <div class="stats-bar">
      <span class="stat"><strong>{projects.length}</strong>&nbsp;projects</span>
      <span class="stat"><strong>{allTasks.length}</strong>&nbsp;tasks</span>
    </div>
    {#if projects.length > 0}
      <select class="ops-select" bind:value={filterProject}>
        <option value="">All projects</option>
        {#each projects as p}
          <option value={p.name}>{p.name}</option>
        {/each}
      </select>
    {/if}
    <div style="display: flex; gap: var(--sp-xxs);">
      <button class="graph-btn" class:active={viewMode === 'kanban'} on:click={() => viewMode = 'kanban'}>⊞ Kanban</button>
      <button class="graph-btn" class:active={viewMode === 'table'} on:click={() => viewMode = 'table'}>☰ Table</button>
    </div>
  </div>
</div>

<div class="main-content">
  {#if loading}
    <div class="loading">Loading operations...</div>
  {:else if projects.length === 0}
    <div class="empty-state">
      <h3>No projects</h3>
      <p>Register a project first:</p>
      <code>base project add --name "My App" --path "src"</code>
    </div>
  {:else}

    {#if viewMode === 'kanban'}
      <div class="kanban">
        {#each [['Active', 'active', 'var(--green)'], ['Blocked', 'blocked', 'var(--red)'], ['Completed', 'completed', 'var(--ink-tertiary)'], ['Pending', 'pending', 'var(--ink-subtle)']] as [label, key, color]}
          <div
            class="kanban-col"
            class:drop-target={dragOverCol === key}
            role="group"
            aria-label="{label} tasks"
            on:dragover={(e) => onDragOver(e, key)}
            on:dragleave={(e) => onDragLeave(e, key)}
            on:drop={(e) => onDrop(e, key)}
          >
            <div class="kanban-col-header">
              <span class="kanban-col-dot" style="background: {color}"></span>
              <span>{label}</span>
              <span class="kanban-col-count">{kanbanColumns[key].length}</span>
            </div>
            <div class="kanban-col-cards">
              {#each kanbanColumns[key] as task (task.iri)}
                <div
                  class="kanban-card"
                  class:dragging={dragIri === task.iri}
                  draggable="true"
                  role="listitem"
                  on:dragstart={(e) => onDragStart(e, task)}
                  on:dragend={onDragEnd}
                >
                  <div class="kanban-card-name">{task.name}</div>
                  <div class="kanban-card-meta">
                    <span class="kanban-card-project">{task.project}</span>
                    {#if task.priority && task.priority !== 'normal'}
                      <span class="kanban-card-priority" style="color: {task.priority === 'high' ? 'var(--orange)' : 'var(--ink-subtle)'}">
                        {task.priority}
                      </span>
                    {/if}
                  </div>
                </div>
              {/each}
              {#if kanbanColumns[key].length === 0}
                <div class="kanban-empty">No tasks</div>
              {/if}
            </div>
          </div>
        {/each}
      </div>

    {:else}
      <div class="ops-table-wrap">
        <table class="ops-table">
          <thead>
            <tr>
              {#each ['name', 'project', 'status', 'priority'] as col}
                <th on:click={() => sortTable(col)} class:sorted={sortCol === col}>
                  {col.charAt(0).toUpperCase() + col.slice(1)}
                  {#if sortCol === col}<span class="sort-arrow">{sortDir > 0 ? '↑' : '↓'}</span>{/if}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each sortedTasks as task, i}
              <tr class:alt={i % 2 === 1}>
                <td>{task.name}</td>
                <td>{task.project}</td>
                <td><span class="status-badge" style="color: {statusColor(task.status)}">{task.status}</span></td>
                <td>{task.priority}</td>
              </tr>
            {/each}
            {#if sortedTasks.length === 0}
              <tr><td colspan="4" style="text-align: center; color: var(--ink-tertiary);">No tasks</td></tr>
            {/if}
          </tbody>
        </table>
      </div>
    {/if}

    <!-- Bottom cards: Decisions, People, Reminders -->
    <div class="ops-bottom-cards">
      <div class="ops-card">
        <h4>Recent Decisions</h4>
        {#if decisions.length > 0}
          {#each decisions.slice(0, 5) as dec}
            <div class="ops-card-row">
              <span class="ops-card-name">{dec.name}</span>
              <span class="ops-card-detail">{dec.rationale.length > 60 ? dec.rationale.slice(0, 60) + '…' : dec.rationale}</span>
            </div>
          {/each}
        {:else}
          <p class="ops-card-empty">No decisions logged</p>
        {/if}
      </div>

      <div class="ops-card">
        <h4>Projects</h4>
        {#each projects as proj}
          <div class="ops-card-row">
            <span class="ops-card-name">{proj.name}</span>
            <span class="status-badge" style="color: {statusColor(proj.status)}">{proj.status}</span>
          </div>
        {/each}
      </div>

      <div class="ops-card">
        <h4>Reminders</h4>
        {#if reminders.length > 0}
          {#each reminders as rem}
            <div class="ops-card-row" class:overdue={rem.overdue}>
              <span class="ops-card-name">{rem.name}</span>
              <span class="ops-card-detail">{rem.due}</span>
            </div>
          {/each}
        {:else}
          <p class="ops-card-empty">No reminders</p>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  :global(.kanban-col.drop-target) {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
    background: rgba(88, 139, 248, 0.05) !important;
  }
  :global(.kanban-card.dragging) {
    opacity: 0.4;
  }
  :global(.kanban-card[draggable="true"]) {
    cursor: grab;
  }
  :global(.kanban-card[draggable="true"]:active) {
    cursor: grabbing;
  }
</style>
