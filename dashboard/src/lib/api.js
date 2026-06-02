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
