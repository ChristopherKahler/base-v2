const BASE = '';

export async function getNodes() {
  try { const r = await fetch(`${BASE}/api/graph/nodes`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function getEdges() {
  try { const r = await fetch(`${BASE}/api/graph/edges`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function searchNodes(query) {
  if (!query) return [];
  try { const r = await fetch(`${BASE}/api/graph/search?q=${encodeURIComponent(query)}`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function getNodeDetail(iri) {
  try { const r = await fetch(`${BASE}/api/graph/node/${encodeURIComponent(iri)}`); return r.ok ? await r.json() : null; }
  catch { return null; }
}

export async function getNotes(iri) {
  try { const r = await fetch(`${BASE}/api/graph/node/${encodeURIComponent(iri)}/notes`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function addNote(iri, text) {
  try {
    const r = await fetch(`${BASE}/api/graph/node/${encodeURIComponent(iri)}/notes`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function updateNote(iri, index, text) {
  try {
    const r = await fetch(`${BASE}/api/graph/node/${encodeURIComponent(iri)}/notes/${index}`, {
      method: 'PUT', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function deleteNote(iri, index) {
  try {
    const r = await fetch(`${BASE}/api/graph/node/${encodeURIComponent(iri)}/notes/${index}`, { method: 'DELETE' });
    return r.ok;
  } catch { return false; }
}

export async function getProjects() {
  try { const r = await fetch(`${BASE}/api/ops/projects`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function getDecisions() {
  try { const r = await fetch(`${BASE}/api/ops/decisions`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function createDecision(name, rationale, domain) {
  try {
    const r = await fetch(`${BASE}/api/ops/decision`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, rationale, domain: domain || undefined }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function updateDecision(iri, fields) {
  try {
    const r = await fetch(`${BASE}/api/ops/decision/${encodeURIComponent(iri)}`, {
      method: 'PATCH', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(fields),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function deleteDecision(iri) {
  try {
    const r = await fetch(`${BASE}/api/ops/decision/${encodeURIComponent(iri)}`, { method: 'DELETE' });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function getReminders() {
  try { const r = await fetch(`${BASE}/api/ops/reminders`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function createReminder(name, due, relatedTo) {
  try {
    const r = await fetch(`${BASE}/api/ops/reminder`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, due, related_to: relatedTo || null }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function completeReminder(iri) {
  try {
    const r = await fetch(`${BASE}/api/ops/reminder/${encodeURIComponent(iri)}/complete`, {
      method: 'PATCH',
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function deleteReminder(iri) {
  try {
    const r = await fetch(`${BASE}/api/ops/reminder/${encodeURIComponent(iri)}`, { method: 'DELETE' });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function getLedger() {
  try { const r = await fetch(`${BASE}/api/ops/ledger`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function getCostSummary(project = '') {
  try { const r = await fetch(`${BASE}/api/ops/cost-summary?project=${encodeURIComponent(project)}`); return r.ok ? await r.json() : null; }
  catch { return null; }
}

export async function updateProjectStatus(iri, status) {
  try {
    const r = await fetch(`${BASE}/api/ops/project/${encodeURIComponent(iri)}/status`, {
      method: 'PATCH', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export function createSessionWs() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return new WebSocket(`${proto}//${location.host}/api/ws/session`);
}

export async function updateTaskStatus(iri, status) {
  try {
    const r = await fetch(`${BASE}/api/ops/task/${encodeURIComponent(iri)}/status`, {
      method: 'PATCH', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function getUsageSummary(days = 30) {
  try { const r = await fetch(`${BASE}/api/usage/summary?days=${days}`); return r.ok ? await r.json() : null; }
  catch { return null; }
}

export async function getUsageSessions(days = 30) {
  try { const r = await fetch(`${BASE}/api/usage/sessions?days=${days}`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function getUsageProjects(days = 30) {
  try { const r = await fetch(`${BASE}/api/usage/projects?days=${days}`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function reloadGraph() {
  try {
    const r = await fetch(`${BASE}/api/graph/reload`, { method: 'POST' });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function createTask(name, project, priority = 'normal', description = '') {
  try {
    const r = await fetch(`${BASE}/api/ops/task`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, project, status: 'active', priority, description: description || undefined }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function updateTask(iri, fields) {
  try {
    const r = await fetch(`${BASE}/api/ops/task/${encodeURIComponent(iri)}`, {
      method: 'PATCH', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(fields),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function deleteTask(iri) {
  try {
    const r = await fetch(`${BASE}/api/ops/task/${encodeURIComponent(iri)}`, { method: 'DELETE' });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function createEntity(name, type, domain) {
  try {
    const r = await fetch(`${BASE}/api/graph/entity`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, type, domain }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function getDomains() {
  try { const r = await fetch(`${BASE}/api/domains`); return r.ok ? await r.json() : []; }
  catch { return []; }
}

export async function addRule(domain, text) {
  try {
    const r = await fetch(`${BASE}/api/domains/rule`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ domain, text }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function deleteRule(domain, text) {
  try {
    const r = await fetch(`${BASE}/api/domains/rule`, {
      method: 'DELETE', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ domain, text }),
    });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export function exportUsageCsv(days = 30) {
  window.open(`${BASE}/api/export/usage-csv?days=${days}`, '_blank');
}

export function exportGraphJson() {
  window.open(`${BASE}/api/export/graph-json`, '_blank');
}
