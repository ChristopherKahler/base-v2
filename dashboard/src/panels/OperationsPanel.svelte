<script>
  import { onMount } from 'svelte';
  import { getProjects, getDecisions, getReminders, getDomains, updateTaskStatus, createTask, updateTask, deleteTask, createDecision, updateDecision, deleteDecision, createReminder, completeReminder, deleteReminder, updateProjectStatus } from '../lib/api.js';

  let projects = [];
  let decisions = [];
  let reminders = [];
  let domains = [];
  let loading = true;
  let viewMode = 'kanban';
  let filterProject = '';
  let showClosedProjects = false;

  // ─── Create Task Modal ────────────────────────────────────
  let showCreateModal = false;
  let newTaskName = '';
  let newTaskProject = '';
  let newTaskPriority = 'normal';
  let newTaskDescription = '';

  function openCreateModal() {
    newTaskName = '';
    newTaskProject = projects.length === 1 ? projects[0].name : '';
    newTaskPriority = 'normal';
    newTaskDescription = '';
    showCreateModal = true;
  }
  function closeCreateModal() { showCreateModal = false; }
  function handleModalKeydown(e) { if (e.key === 'Escape') closeCreateModal(); }
  function handleModalBackdrop(e) { if (e.target === e.currentTarget) closeCreateModal(); }

  async function doCreateTask() {
    if (!newTaskName.trim() || !newTaskProject) return;
    await createTask(newTaskName, newTaskProject, newTaskPriority, newTaskDescription);
    closeCreateModal();
    projects = await getProjects();
  }

  // ─── Task Detail Panel ────────────────────────────────────
  let selectedTask = null;
  let editingName = false;
  let editNameValue = '';
  let showDeleteConfirm = false;

  function selectTask(task) {
    selectedTask = { ...task };
    editingName = false;
    showDeleteConfirm = false;
  }
  function closeDetail() { selectedTask = null; editingName = false; showDeleteConfirm = false; }
  function startEditName() { editNameValue = selectedTask.name; editingName = true; }

  async function saveName() {
    if (!editNameValue.trim() || editNameValue === selectedTask.name) { editingName = false; return; }
    const result = await updateTask(selectedTask.iri, { name: editNameValue.trim() });
    if (result) { updateLocalTask(selectedTask.iri, { name: editNameValue.trim() }); selectedTask = { ...selectedTask, name: editNameValue.trim() }; }
    editingName = false;
  }
  function handleNameKeydown(e) { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') editingName = false; }

  async function changeStatus(e) {
    const s = e.target.value;
    const result = await updateTaskStatus(selectedTask.iri, s);
    if (result) { updateLocalTask(selectedTask.iri, { status: s }); selectedTask = { ...selectedTask, status: s }; }
  }
  async function changePriority(e) {
    const p = e.target.value;
    const result = await updateTask(selectedTask.iri, { priority: p });
    if (result) { updateLocalTask(selectedTask.iri, { priority: p }); selectedTask = { ...selectedTask, priority: p }; }
  }
  async function saveDescription() {
    const desc = selectedTask.description || '';
    await updateTask(selectedTask.iri, { description: desc });
    updateLocalTask(selectedTask.iri, { description: desc });
  }
  async function confirmDelete() {
    const result = await deleteTask(selectedTask.iri);
    if (result) { projects = projects.map(p => ({ ...p, tasks: p.tasks.filter(t => t.iri !== selectedTask.iri) })); closeDetail(); }
  }
  function updateLocalTask(iri, fields) {
    projects = projects.map(p => ({ ...p, tasks: p.tasks.map(t => t.iri === iri ? { ...t, ...fields } : t) }));
  }

  // ─── Decisions ────────────────────────────────────────────
  let expandedDecision = null;
  let showDecisionForm = false;
  let newDecName = '';
  let newDecRationale = '';
  let newDecDomain = '';
  let showAllDecisions = false;
  let editingDecision = null; // iri of decision being edited

  function toggleDecision(iri) { expandedDecision = expandedDecision === iri ? null : iri; }

  async function doCreateDecision() {
    if (!newDecName.trim()) return;
    await createDecision(newDecName, newDecRationale, newDecDomain);
    newDecName = ''; newDecRationale = ''; newDecDomain = '';
    showDecisionForm = false;
    decisions = await getDecisions();
  }
  async function doUpdateDecision(dec) {
    await updateDecision(dec.iri, { name: dec.name, rationale: dec.rationale, domain: dec.domain });
    editingDecision = null;
    decisions = await getDecisions();
  }
  async function doDeleteDecision(iri) {
    await deleteDecision(iri);
    decisions = decisions.filter(d => d.iri !== iri);
    if (expandedDecision === iri) expandedDecision = null;
  }

  // ─── Reminders ────────────────────────────────────────────
  let showReminderForm = false;
  let newRemName = '';
  let newRemDue = '';

  $: activeReminders = reminders.filter(r => r.status !== 'completed');
  $: completedReminders = reminders.filter(r => r.status === 'completed');

  async function doCreateReminder() {
    if (!newRemName.trim()) return;
    await createReminder(newRemName, newRemDue);
    newRemName = ''; newRemDue = '';
    showReminderForm = false;
    reminders = await getReminders();
  }
  async function doCompleteReminder(iri) {
    await completeReminder(iri);
    reminders = reminders.map(r => r.iri === iri ? { ...r, status: 'completed', overdue: false } : r);
  }
  async function doDeleteReminder(iri) {
    await deleteReminder(iri);
    reminders = reminders.filter(r => r.iri !== iri);
  }

  // ─── Projects ─────────────────────────────────────────────
  async function changeProjectStatus(proj, e) {
    const s = e.target.value;
    const result = await updateProjectStatus(proj.iri, s);
    if (result) { projects = projects.map(p => p.iri === proj.iri ? { ...p, status: s } : p); }
  }
  function filterByProject(name) { filterProject = filterProject === name ? '' : name; }

  // ─── Derived ──────────────────────────────────────────────
  $: allTasks = projects.flatMap(p => p.tasks.map(t => ({ ...t, project: p.name, projectStatus: p.status })));
  $: filteredTasks = filterProject ? allTasks.filter(t => t.project === filterProject) : allTasks;
  $: kanbanColumns = {
    active: filteredTasks.filter(t => t.status === 'active' || t.status === 'in_progress'),
    blocked: filteredTasks.filter(t => t.status === 'blocked'),
    completed: filteredTasks.filter(t => t.status === 'completed' || t.status === 'done'),
    pending: filteredTasks.filter(t => !['active','in_progress','blocked','completed','done'].includes(t.status)),
  };
  $: visibleDecisions = showAllDecisions ? decisions : decisions.slice(0, 8);

  let sortCol = 'name'; let sortDir = 1;
  function sortTable(col) { if (sortCol === col) sortDir *= -1; else { sortCol = col; sortDir = 1; } }
  $: sortedTasks = [...filteredTasks].sort((a, b) => (a[sortCol] || '').localeCompare(b[sortCol] || '') * sortDir);

  function statusColor(s) { return { active: 'var(--green)', in_progress: 'var(--green)', blocked: 'var(--red)', completed: 'var(--ink-tertiary)', done: 'var(--ink-tertiary)', deferred: 'var(--ink-subtle)' }[s] || 'var(--ink-subtle)'; }
  function priorityColor(p) { return { urgent: 'var(--red)', high: 'var(--orange)', normal: 'var(--ink-subtle)', low: 'var(--ink-tertiary)' }[p] || 'var(--ink-subtle)'; }

  // ─── Drag-and-Drop ────────────────────────────────────────
  let dragIri = null; let dragOverCol = null; let didDrag = false;
  const statusMap = { active: 'active', blocked: 'blocked', completed: 'completed', pending: 'pending' };
  function onDragStart(e, task) { didDrag = true; dragIri = task.iri; e.dataTransfer.effectAllowed = 'move'; e.dataTransfer.setData('text/plain', task.iri); e.dataTransfer.setDragImage(e.currentTarget, e.offsetX, e.offsetY); }
  function onDragOver(e, k) { e.preventDefault(); e.dataTransfer.dropEffect = 'move'; dragOverCol = k; }
  function onDragLeave(e, k) { if (dragOverCol === k) dragOverCol = null; }
  async function onDrop(e, k) { e.preventDefault(); dragOverCol = null; const iri = dragIri; dragIri = null; if (!iri) return; const s = statusMap[k]; if (!s) return; updateLocalTask(iri, { status: s }); if (selectedTask?.iri === iri) selectedTask = { ...selectedTask, status: s }; const r = await updateTaskStatus(iri, s); if (!r) projects = await getProjects(); }
  function onDragEnd() { dragIri = null; dragOverCol = null; }
  function handleCardClick(task) { if (didDrag) { didDrag = false; return; } selectTask(task); }

  onMount(async () => {
    const [p, d, r, dom] = await Promise.all([getProjects(), getDecisions(), getReminders(), getDomains()]);
    projects = p; decisions = d; reminders = r; domains = dom; loading = false;
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
        {#each projects as p}<option value={p.name}>{p.name}</option>{/each}
      </select>
    {/if}
    <div style="display: flex; gap: var(--sp-xxs);">
      <button class="graph-btn" on:click={openCreateModal}>+ Task</button>
      <button class="graph-btn" class:active={viewMode === 'kanban'} on:click={() => viewMode = 'kanban'}>⊞ Kanban</button>
      <button class="graph-btn" class:active={viewMode === 'table'} on:click={() => viewMode = 'table'}>☰ Table</button>
    </div>
  </div>
</div>

<div class="main-content ops-page">
  {#if loading}
    <div class="loading">Loading operations...</div>
  {:else if projects.length === 0}
    <div class="empty-state"><h3>No projects</h3><p>Register a project first:</p><code>base project add --name "My App" --path "src"</code></div>
  {:else}
    <!-- Task view (kanban or table) -->
    {#if viewMode === 'kanban'}
      <div class="kanban">
        {#each [['Active', 'active', 'var(--green)'], ['Blocked', 'blocked', 'var(--red)'], ['Completed', 'completed', 'var(--ink-tertiary)'], ['Pending', 'pending', 'var(--ink-subtle)']] as [label, key, color]}
          <div class="kanban-col" class:drop-target={dragOverCol === key} role="group" aria-label="{label} tasks"
            on:dragover={(e) => onDragOver(e, key)} on:dragleave={(e) => onDragLeave(e, key)} on:drop={(e) => onDrop(e, key)}>
            <div class="kanban-col-header">
              <span class="kanban-col-dot" style="background: {color}"></span><span>{label}</span>
              <span class="kanban-col-count">{kanbanColumns[key].length}</span>
            </div>
            <div class="kanban-col-cards">
              {#each kanbanColumns[key] as task (task.iri)}
                <div class="kanban-card" class:dragging={dragIri === task.iri} class:selected={selectedTask?.iri === task.iri}
                  draggable="true" role="listitem" on:dragstart={(e) => onDragStart(e, task)} on:dragend={onDragEnd} on:click={() => handleCardClick(task)}>
                  {#if task.priority === 'urgent' || task.priority === 'high'}
                    <div class="kanban-card-priority-bar" style="background: {priorityColor(task.priority)}"></div>
                  {/if}
                  <div class="kanban-card-name">{task.name}</div>
                  <div class="kanban-card-meta">
                    <span class="kanban-card-project">{task.project}</span>
                    {#if task.priority && task.priority !== 'normal'}
                      <span class="kanban-card-priority" style="color: {priorityColor(task.priority)}">{task.priority}</span>
                    {/if}
                  </div>
                  {#if task.description}
                    <div class="kanban-card-desc">{task.description.length > 50 ? task.description.slice(0, 50) + '…' : task.description}</div>
                  {/if}
                </div>
              {/each}
              {#if kanbanColumns[key].length === 0}<div class="kanban-empty">No tasks</div>{/if}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="ops-table-wrap">
        <table class="ops-table">
          <thead><tr>
            {#each ['name', 'project', 'status', 'priority'] as col}
              <th on:click={() => sortTable(col)} class:sorted={sortCol === col}>
                {col.charAt(0).toUpperCase() + col.slice(1)}
                {#if sortCol === col}<span class="sort-arrow">{sortDir > 0 ? '↑' : '↓'}</span>{/if}
              </th>
            {/each}
          </tr></thead>
          <tbody>
            {#each sortedTasks as task, i}
              <tr class:alt={i % 2 === 1} class:selected-row={selectedTask?.iri === task.iri}
                on:click={() => selectTask(task)} style="cursor: pointer;">
                <td>{task.name}</td><td>{task.project}</td>
                <td><span class="status-badge" style="color: {statusColor(task.status)}">{task.status}</span></td>
                <td><span style="color: {priorityColor(task.priority)}">{task.priority}</span></td>
              </tr>
            {/each}
            {#if sortedTasks.length === 0}<tr><td colspan="4" style="text-align: center; color: var(--ink-tertiary);">No tasks</td></tr>{/if}
          </tbody>
        </table>
      </div>
    {/if}

    <!-- Bottom cards -->
    <div class="ops-bottom-cards">

      <!-- DECISIONS -->
      <div class="ops-card">
        <div class="ops-card-header">
          <h4>Decisions <span class="ops-card-count">{decisions.length}</span></h4>
          <button class="ops-card-add" on:click={() => showDecisionForm = !showDecisionForm} title="Log decision">+</button>
        </div>
        {#if showDecisionForm}
          <div class="ops-inline-form">
            <input bind:value={newDecName} placeholder="Decision" class="ops-inline-input" />
            <textarea bind:value={newDecRationale} placeholder="Rationale" class="ops-inline-textarea" rows="2"></textarea>
            <select bind:value={newDecDomain} class="ops-inline-input">
              <option value="">No domain</option>
              {#each domains as d}<option value={d.name}>{d.name}</option>{/each}
            </select>
            <div class="ops-inline-actions">
              <button class="graph-btn" on:click={doCreateDecision} disabled={!newDecName.trim()}>Log</button>
              <button class="graph-btn" on:click={() => showDecisionForm = false}>Cancel</button>
            </div>
          </div>
        {/if}
        {#if decisions.length > 0}
          {#each visibleDecisions as dec (dec.iri)}
            <div class="ops-decision-row" class:expanded={expandedDecision === dec.iri}>
              <div class="ops-decision-summary" on:click={() => toggleDecision(dec.iri)}>
                <span class="ops-decision-name">{dec.name}</span>
                {#if dec.domain}<span class="ops-decision-domain">{dec.domain}</span>{/if}
                <span class="ops-decision-chevron">{expandedDecision === dec.iri ? '▾' : '▸'}</span>
              </div>
              {#if expandedDecision === dec.iri}
                <div class="ops-decision-detail">
                  {#if editingDecision === dec.iri}
                    <input bind:value={dec.name} class="ops-inline-input" placeholder="Decision name" />
                    <textarea bind:value={dec.rationale} class="ops-inline-textarea" rows="3" placeholder="Rationale"></textarea>
                    <select bind:value={dec.domain} class="ops-inline-input">
                      <option value="">No domain</option>
                      {#each domains as d}<option value={d.name}>{d.name}</option>{/each}
                    </select>
                    <div class="ops-inline-actions">
                      <button class="graph-btn" on:click={() => doUpdateDecision(dec)}>Save</button>
                      <button class="graph-btn" on:click={() => editingDecision = null}>Cancel</button>
                    </div>
                  {:else}
                    <p class="ops-decision-rationale">{dec.rationale || 'No rationale'}</p>
                    {#if dec.created_at}<span class="ops-decision-date">{dec.created_at}</span>{/if}
                    <div class="ops-decision-actions">
                      <button class="ops-text-btn" on:click|stopPropagation={() => editingDecision = dec.iri}>Edit</button>
                      <button class="ops-text-btn ops-text-btn-danger" on:click|stopPropagation={() => doDeleteDecision(dec.iri)}>Remove</button>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
          {#if decisions.length > 8}
            <button class="ops-card-toggle" on:click={() => showAllDecisions = !showAllDecisions}>
              {showAllDecisions ? 'Show less' : `Show all ${decisions.length}`}
            </button>
          {/if}
        {:else}
          <p class="ops-card-empty">No decisions logged</p>
        {/if}
      </div>

      <!-- PROJECTS -->
      <div class="ops-card">
        <div class="ops-card-header">
          <h4>Projects</h4>
          <div class="ops-project-toggle">
            <button class="ops-toggle-btn" class:active={!showClosedProjects} on:click={() => showClosedProjects = false}>Open</button>
            <button class="ops-toggle-btn" class:active={showClosedProjects} on:click={() => showClosedProjects = true}>Closed</button>
          </div>
        </div>
        {#each projects.filter(p => showClosedProjects ? (p.status === 'completed' || p.status === 'done') : (p.status !== 'completed' && p.status !== 'done')) as proj}
          {#each [proj.tasks.filter(t => t.status === 'completed' || t.status === 'done').length] as done}
          <div class="ops-project-row" class:active-filter={filterProject === proj.name}
            on:click={() => filterByProject(proj.name)} title="Click to filter tasks">
            <div class="ops-project-info">
              <span class="ops-project-name">{proj.name}</span>
              {#if proj.tasks.length > 0}
                <span class="ops-project-tasks">{done}/{proj.tasks.length}</span>
                <div class="ops-progress-inline">
                  <div class="ops-progress-fill" style="width: {Math.round((done / proj.tasks.length) * 100)}%"></div>
                </div>
              {:else}
                <span class="ops-project-tasks">0 tasks</span>
              {/if}
            </div>
            <div class="ops-project-controls" on:click|stopPropagation>
              <select class="ops-mini-select" value={proj.status} on:change={(e) => changeProjectStatus(proj, e)}>
                <option value="active">active</option>
                <option value="blocked">blocked</option>
                <option value="completed">completed</option>
                <option value="pending">pending</option>
                <option value="deferred">deferred</option>
              </select>
            </div>
          </div>
          {/each}
        {/each}
        {#if projects.filter(p => showClosedProjects ? (p.status === 'completed' || p.status === 'done') : (p.status !== 'completed' && p.status !== 'done')).length === 0}
          <p class="ops-card-empty">{showClosedProjects ? 'No closed projects' : 'No open projects'}</p>
        {/if}
      </div>

      <!-- REMINDERS -->
      <div class="ops-card">
        <div class="ops-card-header">
          <h4>Reminders <span class="ops-card-count">{activeReminders.length}</span></h4>
          <button class="ops-card-add" on:click={() => showReminderForm = !showReminderForm} title="Add reminder">+</button>
        </div>
        {#if showReminderForm}
          <div class="ops-inline-form">
            <input bind:value={newRemName} placeholder="Reminder" class="ops-inline-input" />
            <input bind:value={newRemDue} type="date" class="ops-inline-input" />
            <div class="ops-inline-actions">
              <button class="graph-btn" on:click={doCreateReminder} disabled={!newRemName.trim()}>Add</button>
              <button class="graph-btn" on:click={() => showReminderForm = false}>Cancel</button>
            </div>
          </div>
        {/if}
        {#if activeReminders.length > 0}
          {#each activeReminders as rem}
            <div class="ops-reminder-row" class:overdue={rem.overdue}>
              <span class="ops-reminder-name">{rem.name}</span>
              <span class="ops-reminder-due">{rem.due || 'No date'}</span>
              <div class="ops-reminder-actions">
                <button class="ops-reminder-complete" on:click={() => doCompleteReminder(rem.iri)} title="Mark complete">✓</button>
                <button class="ops-reminder-dismiss" on:click={() => doDeleteReminder(rem.iri)} title="Dismiss">✕</button>
              </div>
            </div>
          {/each}
        {:else}
          <p class="ops-card-empty">No active reminders</p>
        {/if}
        {#if completedReminders.length > 0}
          <div class="ops-completed-section">
            <span class="ops-completed-label">Completed ({completedReminders.length})</span>
            {#each completedReminders as rem}
              <div class="ops-reminder-row completed">
                <span class="ops-reminder-name">{rem.name}</span>
                <button class="ops-reminder-dismiss" on:click={() => doDeleteReminder(rem.iri)} title="Remove">✕</button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- Task Detail Panel -->
{#if selectedTask}
  <div class="task-detail-panel">
    <div class="task-detail-header">
      <span class="task-detail-title">Task Detail</span>
      <button class="task-detail-close" on:click={closeDetail}>✕</button>
    </div>
    <div class="task-detail-body">
      <div class="task-detail-field">
        <label>Name</label>
        {#if editingName}
          <input class="task-detail-input" bind:value={editNameValue} on:blur={saveName} on:keydown={handleNameKeydown} autofocus />
        {:else}
          <div class="task-detail-editable" on:click={startEditName} title="Click to edit">{selectedTask.name}</div>
        {/if}
      </div>
      <div class="task-detail-field"><label>Project</label><div class="task-detail-value">{selectedTask.project}</div></div>
      <div class="task-detail-field">
        <label>Status</label>
        <select class="task-detail-select" value={selectedTask.status} on:change={changeStatus}>
          <option value="active">Active</option><option value="in_progress">In Progress</option>
          <option value="blocked">Blocked</option><option value="completed">Completed</option><option value="pending">Pending</option>
        </select>
      </div>
      <div class="task-detail-field">
        <label>Priority</label>
        <select class="task-detail-select" value={selectedTask.priority} on:change={changePriority}>
          <option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option><option value="urgent">Urgent</option>
        </select>
      </div>
      <div class="task-detail-field">
        <label>Description</label>
        <textarea class="task-detail-textarea" bind:value={selectedTask.description} on:blur={saveDescription} placeholder="Add a description..." rows="4"></textarea>
      </div>
      <div class="task-detail-danger">
        {#if showDeleteConfirm}
          <div class="task-detail-confirm">
            <span>Delete this task permanently?</span>
            <div class="task-detail-confirm-actions">
              <button class="task-detail-confirm-cancel" on:click={() => showDeleteConfirm = false}>Cancel</button>
              <button class="task-detail-confirm-delete" on:click={confirmDelete}>Delete</button>
            </div>
          </div>
        {:else}
          <button class="task-detail-delete-btn" on:click={() => showDeleteConfirm = true}>🗑 Delete Task</button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- Create Task Modal -->
{#if showCreateModal}
  <div class="task-modal-backdrop" on:click={handleModalBackdrop} on:keydown={handleModalKeydown}>
    <div class="task-modal">
      <div class="task-modal-header"><h3>Create Task</h3><button class="task-detail-close" on:click={closeCreateModal}>✕</button></div>
      <div class="task-modal-body">
        <div class="task-modal-field">
          <label>Name <span class="required">*</span></label>
          <input class="task-detail-input" bind:value={newTaskName} placeholder="What needs to be done?" autofocus
            on:keydown={(e) => { if (e.key === 'Enter' && newTaskName.trim() && newTaskProject) doCreateTask(); }} />
        </div>
        <div class="task-modal-field"><label>Project <span class="required">*</span></label>
          <select class="task-detail-select" bind:value={newTaskProject}><option value="">Select project</option>{#each projects as p}<option value={p.name}>{p.name}</option>{/each}</select></div>
        <div class="task-modal-field"><label>Priority</label>
          <select class="task-detail-select" bind:value={newTaskPriority}><option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option><option value="urgent">Urgent</option></select></div>
        <div class="task-modal-field"><label>Description</label>
          <textarea class="task-detail-textarea" bind:value={newTaskDescription} placeholder="Optional description..." rows="3"></textarea></div>
      </div>
      <div class="task-modal-actions">
        <button class="graph-btn" on:click={closeCreateModal}>Cancel</button>
        <button class="graph-btn task-modal-create" on:click={doCreateTask} disabled={!newTaskName.trim() || !newTaskProject}>Create Task</button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.kanban-col.drop-target) { outline: 2px solid var(--primary); outline-offset: -2px; background: rgba(88, 139, 248, 0.05) !important; }
  :global(.kanban-card.dragging) { opacity: 0.4; }
  :global(.kanban-card[draggable="true"]) { cursor: grab; }
  :global(.kanban-card[draggable="true"]:active) { cursor: grabbing; }
  :global(.kanban-card.selected) { border-color: var(--primary); background: rgba(88, 139, 248, 0.08); }
  :global(.ops-select option) { background: #15171C; color: #ffffff; }
</style>
