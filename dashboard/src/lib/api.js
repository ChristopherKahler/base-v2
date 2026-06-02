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

export async function getReminders() {
  try { const r = await fetch(`${BASE}/api/ops/reminders`); return r.ok ? await r.json() : []; }
  catch { return []; }
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

export async function reloadGraph() {
  try {
    const r = await fetch(`${BASE}/api/graph/reload`, { method: 'POST' });
    return r.ok ? await r.json() : null;
  } catch { return null; }
}

export async function createTask(name, project, status = 'active') {
  try {
    const r = await fetch(`${BASE}/api/ops/task`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, project, status }),
    });
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
