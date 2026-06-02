<script>
  import { onMount, onDestroy } from 'svelte';
  import * as d3 from 'd3';
  import { getNodes, getEdges, getNodeDetail, addNote, updateNote, deleteNote } from '../lib/api.js';

  let container;
  let nodes = [];
  let edges = [];
  let loading = true;
  let searchQuery = '';
  let selectedNode = null;
  let nodeDetail = null;
  let simulation;
  let svgEl;
  let zoomBehavior;
  let gEl;
  let detailEl;
  let noteInput = '';
  let submittingNote = false;
  let editingNoteIndex = null;
  let editingNoteText = '';

  async function refreshGraph() {
    const [n, e] = await Promise.all([getNodes(), getEdges()]);
    nodes = n;
    edges = e;
    const types = new Set(nodes.map(n => n.type));
    allTypes = [...types].sort();
    buildGraph(nodes, edges);
  }

  async function submitNote() {
    if (!noteInput.trim() || !selectedNode || submittingNote) return;
    submittingNote = true;
    const result = await addNote(selectedNode.iri, noteInput.trim());
    if (result && nodeDetail) {
      nodeDetail.notes = [...(nodeDetail.notes || []), result];
      nodeDetail = nodeDetail;
    }
    noteInput = '';
    submittingNote = false;
    refreshGraph();
  }

  function startEditNote(note) {
    editingNoteIndex = note.index;
    editingNoteText = note.text;
  }

  async function saveEditNote() {
    if (!editingNoteText.trim() || !selectedNode) return;
    const result = await updateNote(selectedNode.iri, editingNoteIndex, editingNoteText.trim());
    if (result && nodeDetail) {
      nodeDetail.notes = nodeDetail.notes.map(n =>
        n.index === editingNoteIndex ? { ...n, text: result.text } : n
      );
      nodeDetail = nodeDetail;
    }
    editingNoteIndex = null;
    editingNoteText = '';
  }

  async function removeNote(index) {
    if (!selectedNode) return;
    const ok = await deleteNote(selectedNode.iri, index);
    if (ok && nodeDetail) {
      nodeDetail.notes = nodeDetail.notes.filter(n => n.index !== index);
      nodeDetail = nodeDetail;
    }
    refreshGraph();
  }

  function clearAllFilters() {
    activeFilters = new Set();
    updateVisibility();
  }

  function selectAllFilters() {
    activeFilters = new Set(allTypes);
    updateVisibility();
  }

  // Entity type config
  const typeConfig = {
    Project:   { color: 'var(--entity-project)',   raw: '#00CA53' },
    Milestone: { color: 'var(--entity-milestone)', raw: '#95EFFF' },
    Task:      { color: 'var(--entity-task)',       raw: '#47D18C' },
    Person:    { color: 'var(--entity-person)',     raw: '#F58041' },
    Decision:  { color: 'var(--entity-decision)',   raw: '#FF990A' },
    Document:  { color: 'var(--entity-document)',   raw: '#BF6AFB' },
    Domain:    { color: 'var(--entity-domain)',      raw: '#FF4D89' },
    Rule:      { color: 'var(--entity-rule)',        raw: '#DCDBFF' },
    PaulProject: { color: 'var(--entity-project)', raw: '#00CA53' },
    Function:  { color: 'var(--entity-code)',       raw: '#588BF8' },
    Struct:    { color: 'var(--entity-code)',        raw: '#588BF8' },
    Class:     { color: 'var(--entity-code)',        raw: '#588BF8' },
    Module:    { color: 'var(--entity-code)',        raw: '#5CACF8' },
  };

  const defaultConfig = { color: 'var(--entity-default)', raw: '#68686A' };

  function getTypeColor(type) {
    return (typeConfig[type] || defaultConfig).raw;
  }

  // Filter state
  let activeFilters = new Set(Object.keys(typeConfig));
  let allTypes = [];

  function toggleFilter(type) {
    if (activeFilters.has(type)) {
      activeFilters.delete(type);
    } else {
      activeFilters.add(type);
    }
    activeFilters = activeFilters; // trigger reactivity
    updateVisibility();
  }

  function updateVisibility() {
    if (!container) return;
    const svg = d3.select(container).select('svg');

    svg.selectAll('.node-group')
      .style('display', d => {
        const matchesFilter = activeFilters.has(d.type);
        const matchesSearch = !searchQuery ||
          d.name.toLowerCase().includes(searchQuery.toLowerCase());
        return matchesFilter && matchesSearch ? null : 'none';
      });

    svg.selectAll('.edge-line')
      .style('display', d => {
        const sVisible = activeFilters.has(d.source.type);
        const tVisible = activeFilters.has(d.target.type);
        const sSearch = !searchQuery || d.source.name.toLowerCase().includes(searchQuery.toLowerCase());
        const tSearch = !searchQuery || d.target.name.toLowerCase().includes(searchQuery.toLowerCase());
        return (sVisible && sSearch && tVisible && tSearch) ? null : 'none';
      });
  }

  $: if (searchQuery !== undefined) updateVisibility();

  async function selectNode(node) {
    selectedNode = node;
    nodeDetail = await getNodeDetail(node.iri);
  }

  function closeDetail() {
    selectedNode = null;
    nodeDetail = null;
  }

  function buildGraph(graphNodes, graphEdges) {
    if (!container) return;

    const width = container.clientWidth;
    const height = container.clientHeight;

    // Clean up previous
    d3.select(container).selectAll('svg').remove();

    const svg = d3.select(container)
      .append('svg')
      .attr('width', width)
      .attr('height', height);
    svgEl = svg;

    // Zoom
    const g = svg.append('g');
    gEl = g;
    const zoom = d3.zoom()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => g.attr('transform', event.transform));
    zoomBehavior = zoom;
    svg.call(zoom);

    // Map IRIs to indices for d3-force
    const nodeMap = new Map();
    const simNodes = graphNodes.map((n, i) => {
      nodeMap.set(n.iri, i);
      return { ...n, index: i };
    });

    const simEdges = graphEdges
      .filter(e => nodeMap.has(e.source) && nodeMap.has(e.target))
      .map(e => ({
        source: nodeMap.get(e.source),
        target: nodeMap.get(e.target),
        predicate: e.predicate,
      }));

    // Count connections per node
    const connectionCount = new Map();
    simEdges.forEach(e => {
      connectionCount.set(e.source, (connectionCount.get(e.source) || 0) + 1);
      connectionCount.set(e.target, (connectionCount.get(e.target) || 0) + 1);
    });

    // Scale forces based on node count — tight for small graphs, looser for large
    const n = simNodes.length;
    const linkDist = n < 30 ? 40 : n < 100 ? 55 : 70;
    const chargeStr = n < 30 ? -60 : n < 100 ? -80 : -100;

    simulation = d3.forceSimulation(simNodes)
      .force('link', d3.forceLink(simEdges).distance(linkDist).strength(0.5))
      .force('charge', d3.forceManyBody().strength(chargeStr).distanceMax(300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collide', d3.forceCollide().radius(d => nodeRadius(d, connectionCount) + 3))
      .force('x', d3.forceX(width / 2).strength(0.05))
      .force('y', d3.forceY(height / 2).strength(0.05));

    // Edges
    const link = g.append('g')
      .selectAll('line')
      .data(simEdges)
      .join('line')
      .attr('class', 'edge-line')
      .attr('stroke', '#2D2F31')
      .attr('stroke-width', 1)
      .attr('stroke-opacity', 0.5);

    // Node groups
    const nodeGroup = g.append('g')
      .selectAll('g')
      .data(simNodes)
      .join('g')
      .attr('class', 'node-group')
      .style('cursor', 'pointer')
      .call(d3.drag()
        .on('start', dragStart)
        .on('drag', dragging)
        .on('end', dragEnd))
      .on('click', (event, d) => {
        event.stopPropagation();
        selectNode(d);
      });

    // Node circles
    nodeGroup.append('circle')
      .attr('r', d => nodeRadius(d, connectionCount))
      .attr('fill', d => getTypeColor(d.type))
      .attr('stroke', 'none')
      .attr('opacity', 0.85);

    // Subtle glow on hover
    nodeGroup
      .on('mouseenter', function(event, d) {
        d3.select(this).select('circle')
          .transition().duration(150)
          .attr('opacity', 1)
          .attr('stroke', getTypeColor(d.type))
          .attr('stroke-width', 2)
          .attr('stroke-opacity', 0.4);
        d3.select(this).select('text')
          .transition().duration(150)
          .style('opacity', 1);
      })
      .on('mouseleave', function() {
        d3.select(this).select('circle')
          .transition().duration(150)
          .attr('opacity', 0.85)
          .attr('stroke', 'none');
        d3.select(this).select('text')
          .transition().duration(150)
          .style('opacity', d => {
            const count = connectionCount.get(d.index) || 0;
            return count >= 3 ? 0.9 : 0;
          });
      });

    // Labels (visible for well-connected nodes)
    nodeGroup.append('text')
      .text(d => d.name || d.iri.split('/').pop())
      .attr('dy', d => nodeRadius(d, connectionCount) + 14)
      .attr('text-anchor', 'middle')
      .attr('fill', 'var(--ink-muted)')
      .attr('font-size', '11px')
      .attr('font-weight', '400')
      .style('pointer-events', 'none')
      .style('opacity', d => {
        const count = connectionCount.get(d.index) || 0;
        return count >= 3 ? 0.9 : 0;
      });

    // Deselect on background click
    svg.on('click', () => closeDetail());

    // Tick
    simulation.on('tick', () => {
      link
        .attr('x1', d => d.source.x)
        .attr('y1', d => d.source.y)
        .attr('x2', d => d.target.x)
        .attr('y2', d => d.target.y);

      nodeGroup.attr('transform', d => `translate(${d.x},${d.y})`);
    });

    // Initial zoom to fit
    setTimeout(() => {
      const bounds = g.node().getBBox();
      if (bounds.width > 0 && bounds.height > 0) {
        const fullWidth = width;
        const fullHeight = height;
        const bWidth = bounds.width;
        const bHeight = bounds.height;
        const scale = 0.8 * Math.min(fullWidth / bWidth, fullHeight / bHeight);
        const tx = fullWidth / 2 - scale * (bounds.x + bWidth / 2);
        const ty = fullHeight / 2 - scale * (bounds.y + bHeight / 2);
        svg.transition().duration(500)
          .call(zoom.transform, d3.zoomIdentity.translate(tx, ty).scale(scale));
      }
    }, 1500);
  }

  function startResize(e) {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const startWidth = detailEl.offsetWidth;

    function onMove(ev) {
      // Dragging left = bigger panel (since panel is right-anchored)
      const delta = startX - ev.clientX;
      const newWidth = Math.max(280, Math.min(startWidth + delta, window.innerWidth * 0.6));
      detailEl.style.width = newWidth + 'px';
    }

    function onUp() {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function recenter() {
    if (!svgEl || !gEl || !zoomBehavior) return;
    const bounds = gEl.node().getBBox();
    if (bounds.width === 0 || bounds.height === 0) return;
    const width = container.clientWidth;
    const height = container.clientHeight;
    const scale = 0.8 * Math.min(width / bounds.width, height / bounds.height);
    const tx = width / 2 - scale * (bounds.x + bounds.width / 2);
    const ty = height / 2 - scale * (bounds.y + bounds.height / 2);
    svgEl.transition().duration(400)
      .call(zoomBehavior.transform, d3.zoomIdentity.translate(tx, ty).scale(scale));
  }

  function tighten() {
    if (!simulation) return;
    // Reheat simulation to pull nodes closer
    simulation.alpha(0.8).restart();
    // Zoom to fit after settling
    setTimeout(recenter, 1200);
  }

  function nodeRadius(d, connectionCount) {
    const count = connectionCount.get(d.index) || 0;
    return Math.min(4 + count * 1.5, 18);
  }

  function dragStart(event) {
    if (!event.active) simulation.alphaTarget(0.3).restart();
    event.subject.fx = event.subject.x;
    event.subject.fy = event.subject.y;
  }

  function dragging(event) {
    event.subject.fx = event.x;
    event.subject.fy = event.y;
  }

  function dragEnd(event) {
    if (!event.active) simulation.alphaTarget(0);
    event.subject.fx = null;
    event.subject.fy = null;
  }

  onMount(async () => {
    const [n, e] = await Promise.all([getNodes(), getEdges()]);
    nodes = n;
    edges = e;

    // Collect all types
    const types = new Set(nodes.map(n => n.type));
    allTypes = [...types].sort();
    activeFilters = new Set(allTypes);

    loading = false;

    if (nodes.length > 0) {
      // Wait for container to render
      await new Promise(r => setTimeout(r, 50));
      buildGraph(nodes, edges);
    }
  });

  onDestroy(() => {
    if (simulation) simulation.stop();
  });
</script>

<div class="main-header">
  <h2>Graph Explorer</h2>
  <div style="display: flex; align-items: center; gap: var(--sp-md);">
    <div class="stats-bar">
      <span class="stat"><strong>{nodes.length}</strong>&nbsp;nodes</span>
      <span class="stat"><strong>{edges.length}</strong>&nbsp;edges</span>
    </div>
    <div style="display: flex; gap: var(--sp-xxs);">
      <button class="graph-btn" on:click={tighten} title="Tighten layout &amp; re-center">⊛ Align</button>
      <button class="graph-btn" on:click={recenter} title="Zoom to fit all nodes">◎ Fit</button>
    </div>
    <div class="search-bar">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--ink-tertiary)" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
      </svg>
      <input
        type="text"
        placeholder="Search nodes..."
        bind:value={searchQuery}
      />
    </div>
  </div>
</div>

<div class="main-content" style="display: flex; flex-direction: column; gap: var(--sp-md); padding-bottom: 0;">
  {#if loading}
    <div class="loading">Loading graph...</div>
  {:else if nodes.length === 0}
    <div class="empty-state">
      <h3>No graph data</h3>
      <p>Populate your knowledge graph first:</p>
      <code>base sync</code>
      <p style="margin-top: var(--sp-xs);">Then restart the dashboard.</p>
    </div>
  {:else}
    <div class="filter-bar">
      <button class="graph-btn" on:click={clearAllFilters} style="font-size: 11px; padding: 2px 8px;">Clear</button>
      <button class="graph-btn" on:click={selectAllFilters} style="font-size: 11px; padding: 2px 8px;">All</button>
      {#each allTypes as type}
        <button
          class="filter-chip"
          class:active={activeFilters.has(type)}
          on:click={() => toggleFilter(type)}
        >
          <span class="chip-dot" style="background: {getTypeColor(type)}"></span>
          {type}
        </button>
      {/each}
    </div>

    <div class="graph-container" bind:this={container}>
      {#if selectedNode && nodeDetail}
        <div class="detail-panel" bind:this={detailEl}>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="detail-resize" on:mousedown={startResize}></div>
          <button class="detail-close" on:click|stopPropagation={closeDetail}>✕</button>
          <h3>{nodeDetail.name || selectedNode.name}</h3>
          <span
            class="detail-type"
            style="background: {getTypeColor(nodeDetail.type)}22; color: {getTypeColor(nodeDetail.type)};"
          >
            {nodeDetail.type}
          </span>

          {#if nodeDetail.properties && Object.keys(nodeDetail.properties).length > 0}
            <div class="detail-section">
              <h4>Properties</h4>
              {#each Object.entries(nodeDetail.properties).filter(([k]) => k !== 'name') as [key, value]}
                <div class="detail-row">
                  <span class="label">{key}</span>
                  <span class="value">{value}</span>
                </div>
              {/each}
            </div>
          {/if}

          {#if nodeDetail.outgoing && nodeDetail.outgoing.length > 0}
            <div class="detail-section">
              <h4>Outgoing ({nodeDetail.outgoing.length})</h4>
              {#each nodeDetail.outgoing.slice(0, 15) as edge}
                <div class="detail-edge">
                  <span class="pred">{edge.predicate} →</span>
                  <span>{edge.target.split('/').pop()}</span>
                </div>
              {/each}
              {#if nodeDetail.outgoing.length > 15}
                <div class="detail-edge" style="color: var(--ink-tertiary);">
                  +{nodeDetail.outgoing.length - 15} more
                </div>
              {/if}
            </div>
          {/if}

          {#if nodeDetail.incoming && nodeDetail.incoming.length > 0}
            <div class="detail-section">
              <h4>Incoming ({nodeDetail.incoming.length})</h4>
              {#each nodeDetail.incoming.slice(0, 15) as edge}
                <div class="detail-edge">
                  <span>{edge.source.split('/').pop()}</span>
                  <span class="pred">→ {edge.predicate}</span>
                </div>
              {/each}
              {#if nodeDetail.incoming.length > 15}
                <div class="detail-edge" style="color: var(--ink-tertiary);">
                  +{nodeDetail.incoming.length - 15} more
                </div>
              {/if}
            </div>
          {/if}

          <div class="detail-section notes-section">
            <h4>Notes {#if nodeDetail.notes?.length > 0}({nodeDetail.notes.length}){/if}</h4>
            {#if nodeDetail.notes?.length > 0}
              {#each nodeDetail.notes as note}
                <div class="note-entry">
                  <span class="note-index">#{note.index}</span>
                  {#if editingNoteIndex === note.index}
                    <input
                      class="note-edit-input"
                      bind:value={editingNoteText}
                      on:keydown={(e) => { if (e.key === 'Enter') saveEditNote(); if (e.key === 'Escape') { editingNoteIndex = null; } }}
                    />
                    <button class="note-action" on:click={saveEditNote} title="Save">✓</button>
                  {:else}
                    <span class="note-text">{note.text}</span>
                    {#if note.created_at}
                      <span class="note-time">{note.created_at.slice(0, 10)}</span>
                    {/if}
                    <button class="note-action" on:click={() => startEditNote(note)} title="Edit">✎</button>
                    <button class="note-action delete" on:click={() => removeNote(note.index)} title="Delete">✕</button>
                  {/if}
                </div>
              {/each}
            {:else}
              <p style="color: var(--ink-tertiary); font-size: 12px;">No notes yet</p>
            {/if}
            <div class="note-input-row">
              <input
                type="text"
                class="note-input"
                placeholder="Add a note..."
                bind:value={noteInput}
                on:keydown={(e) => e.key === 'Enter' && submitNote()}
                disabled={submittingNote}
              />
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
