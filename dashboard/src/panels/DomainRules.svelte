<script>
  import { onMount } from 'svelte';
  import { getDomains, addRule, deleteRule } from '../lib/api.js';

  let domains = [];
  let loading = true;
  let addingTo = null;
  let newRuleText = '';
  let editingDomain = null;
  let editingIndex = null;
  let editingText = '';

  async function reload() {
    domains = await getDomains();
  }

  async function doAddRule(domainName) {
    if (!newRuleText.trim()) return;
    await addRule(domainName, newRuleText.trim());
    newRuleText = '';
    addingTo = null;
    await reload();
  }

  async function doDeleteRule(domainName, ruleText) {
    await deleteRule(domainName, ruleText);
    await reload();
  }

  function startEdit(domainName, index, text) {
    editingDomain = domainName;
    editingIndex = index;
    editingText = text;
  }

  async function saveEdit(domainName, oldText) {
    if (!editingText.trim() || editingText.trim() === oldText) {
      editingDomain = null;
      editingIndex = null;
      return;
    }
    await deleteRule(domainName, oldText);
    await addRule(domainName, editingText.trim());
    editingDomain = null;
    editingIndex = null;
    editingText = '';
    await reload();
  }

  function cancelEdit() {
    editingDomain = null;
    editingIndex = null;
    editingText = '';
  }

  onMount(async () => {
    await reload();
    loading = false;
  });
</script>

<div class="main-header">
  <h2>Domain Rules</h2>
  <span class="header-meta">{domains.length} domain{domains.length !== 1 ? 's' : ''} · {domains.reduce((a, d) => a + d.rules.length, 0)} rules</span>
</div>

<div class="main-content">
  {#if loading}
    <div class="loading">Loading domains...</div>
  {:else if domains.length === 0}
    <div class="empty-state">
      <h3>No domains configured</h3>
      <p>Add domains to domains.toml in your workspace.</p>
    </div>
  {:else}
    <div class="domain-list">
      {#each domains as domain}
        <div class="domain-card">
          <div class="domain-header">
            <span class="domain-name">{domain.name}</span>
            <span class="mode-badge" class:always={domain.mode === 'always'}>{domain.mode}</span>
            <span class="spacer"></span>
            <button class="add-btn" on:click={() => { addingTo = addingTo === domain.name ? null : domain.name; newRuleText = ''; }}>
              {addingTo === domain.name ? '✕' : '+ Rule'}
            </button>
          </div>

          {#if domain.prompt_keywords.length > 0}
            <div class="domain-section">
              <span class="section-label">Keywords</span>
              <div class="chip-row">
                {#each domain.prompt_keywords as kw}
                  <span class="kw-chip">{kw}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if domain.paths.length > 0}
            <div class="domain-section">
              <span class="section-label">Paths</span>
              <div class="chip-row">
                {#each domain.paths as p}
                  <span class="path-chip">{p}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if addingTo === domain.name}
            <div class="add-rule-form">
              <input
                bind:value={newRuleText}
                placeholder="New rule text..."
                class="rule-input"
                on:keydown={(e) => e.key === 'Enter' && doAddRule(domain.name)}
              />
              <button class="add-btn" on:click={() => doAddRule(domain.name)} disabled={!newRuleText.trim()}>Add</button>
            </div>
          {/if}

          {#if domain.rules.length > 0}
            <div class="domain-section">
              <span class="section-label">Rules ({domain.rules.length})</span>
              <div class="rules-list">
                {#each domain.rules as rule, i}
                  <div class="rule-row">
                    <span class="rule-num">{i}</span>
                    {#if editingDomain === domain.name && editingIndex === i}
                      <input
                        class="rule-edit-input"
                        bind:value={editingText}
                        on:keydown={(e) => { if (e.key === 'Enter') saveEdit(domain.name, rule); if (e.key === 'Escape') cancelEdit(); }}
                      />
                      <button class="rule-action" on:click={() => saveEdit(domain.name, rule)} title="Save">✓</button>
                      <button class="rule-action" on:click={cancelEdit} title="Cancel">✕</button>
                    {:else}
                      <span class="rule-text" on:dblclick={() => startEdit(domain.name, i, rule)}>{rule}</span>
                      <button class="rule-edit" on:click={() => startEdit(domain.name, i, rule)} title="Edit rule">✎</button>
                      <button class="rule-delete" on:click={() => doDeleteRule(domain.name, rule)} title="Delete rule">✕</button>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <div class="domain-section">
              <span class="section-label">Rules</span>
              <span class="no-rules">No rules configured</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .main-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 24px; border-bottom: 1px solid var(--border);
  }
  .main-header h2 { margin: 0; font-size: 15px; font-weight: 600; color: var(--ink-primary); }
  .header-meta { font-size: 11px; color: var(--ink-tertiary); }

  .main-content { flex: 1; overflow-y: auto; padding: 12px 24px; }
  .loading, .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    height: 200px; color: var(--ink-tertiary);
  }
  .empty-state h3 { margin: 0 0 4px; font-size: 15px; }
  .empty-state p { margin: 0; font-size: 12px; }

  .domain-list { display: flex; flex-direction: column; gap: 10px; }

  .domain-card {
    background: var(--surface-02); border: 1px solid var(--border);
    border-radius: 8px; padding: 14px 16px;
  }
  .domain-header {
    display: flex; align-items: center; gap: 8px; margin-bottom: 10px;
  }
  .domain-name { font-size: 14px; font-weight: 600; color: var(--ink-primary); }
  .spacer { flex: 1; }
  .mode-badge {
    font-size: 9px; font-weight: 600; letter-spacing: 0.4px;
    padding: 1px 6px; border-radius: 3px;
    background: var(--surface-03); color: var(--ink-tertiary);
    text-transform: uppercase;
  }
  .mode-badge.always { background: rgba(0, 202, 83, 0.15); color: var(--green); }
  .add-btn {
    font-size: 11px; padding: 2px 8px; border-radius: 4px;
    background: var(--surface-03); border: 1px solid var(--border);
    color: var(--ink-secondary); cursor: pointer;
  }
  .add-btn:hover { color: var(--ink-primary); background: var(--surface-04, var(--surface-03)); }

  .domain-section { margin-top: 8px; }
  .section-label {
    display: block; font-size: 10px; font-weight: 600; color: var(--ink-tertiary);
    text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px;
  }
  .chip-row { display: flex; flex-wrap: wrap; gap: 4px; }
  .kw-chip {
    font-size: 11px; padding: 1px 7px; border-radius: 8px;
    background: rgba(88, 139, 248, 0.1); color: var(--primary);
    border: 1px solid rgba(88, 139, 248, 0.15);
  }
  .path-chip {
    font-size: 11px; padding: 1px 7px; border-radius: 8px;
    background: rgba(245, 128, 65, 0.1); color: var(--orange);
    border: 1px solid rgba(245, 128, 65, 0.15);
    font-family: monospace;
  }

  .add-rule-form {
    display: flex; gap: 6px; margin-top: 8px; padding: 8px;
    background: var(--surface-03); border-radius: 6px;
  }
  .rule-input {
    flex: 1; background: var(--canvas); border: 1px solid var(--border);
    color: var(--ink-primary); padding: 4px 8px; border-radius: 4px;
    font-size: 12px;
  }

  .rules-list { display: flex; flex-direction: column; gap: 2px; }
  .rule-row {
    display: flex; gap: 8px; padding: 4px 0; align-items: flex-start;
    position: relative;
  }
  .rule-row:hover .rule-delete { opacity: 1; }
  .rule-num {
    font-size: 10px; color: var(--ink-tertiary); min-width: 14px;
    text-align: right; font-variant-numeric: tabular-nums; padding-top: 2px;
  }
  .rule-text { font-size: 12px; color: var(--ink-secondary); line-height: 1.4; flex: 1; }
  .rule-edit, .rule-delete {
    opacity: 0; font-size: 10px; background: none;
    border: none; cursor: pointer; padding: 2px 4px; transition: opacity 0.15s;
  }
  .rule-edit { color: var(--ink-secondary); }
  .rule-delete { color: var(--red); }
  .rule-row:hover .rule-edit, .rule-row:hover .rule-delete { opacity: 1; }
  .rule-edit-input {
    flex: 1; background: var(--canvas); border: 1px solid var(--primary);
    color: var(--ink-primary); padding: 3px 8px; border-radius: 4px;
    font-size: 12px; outline: none;
  }
  .rule-action {
    font-size: 11px; background: none; border: none;
    cursor: pointer; padding: 2px 4px; color: var(--ink-secondary);
  }
  .rule-action:first-of-type { color: var(--green); }
  .rule-text { cursor: default; }
  .no-rules { font-size: 12px; color: var(--ink-tertiary); font-style: italic; }
</style>
