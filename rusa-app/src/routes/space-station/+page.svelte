<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { stationApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'inventory' | 'annotations' | 'personnel' | 'findings' | 'supply' | 'abandonment';
  let activeTab = $state<Tab>('inventory');

  let stations: any[] = $state([]);
  let inventory: any[] = $state([]);
  let annotations: any[] = $state([]);
  let personnelLog: any[] = $state([]);
  let findings: any[] = $state([]);
  let supplyRequests: any[] = $state([]);
  let allUsers: any[] = $state([]);
  let selectedStation = $state('');
  let loading = $state(false);

  // Inventory form
  let invOpen = $state(false);
  let invCategory = $state('');
  let invItem = $state('');
  let invQty = $state(0);
  let invUnit = $state('');

  // Annotation form
  let annotOpen = $state(false);
  let annotSection = $state('');
  let annotDesc = $state('');

  // Findings form
  let findOpen = $state(false);
  let findTitle = $state('');
  let findDesc = $state('');
  let findPrivate = $state(false);

  // Personnel form
  let personnelOpen = $state(false);
  let personnelUserSelected: any = $state(null);
  let personnelEventType = $state('');
  let personnelNotes = $state('');

  // Supply request form
  let supplyOpen = $state(false);
  let supplyTitle = $state('');
  let supplyItems = $state('');
  let supplyCost = $state('');

  // Supply review modal
  let supplyReviewOpen = $state(false);
  let supplyReviewTarget: any = $state(null);
  let supplyReviewDecision = $state('');
  let supplyReviewNotes = $state('');

  // Abandonment form
  let abandonOpen = $state(false);
  let abandonReason = $state('');

  const isDirector = $derived(($session?.tier ?? 0) >= 3);

  const personnelEventOpts = [
    { value: 'arrival', label: 'Arrival' },
    { value: 'departure', label: 'Departure' },
    { value: 'transfer', label: 'Transfer' },
    { value: 'incident', label: 'Incident' },
    { value: 'other', label: 'Other' },
  ];

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [stations, allUsers] = await Promise.all([
        stationApi.getStations(s.token),
        userApi.getAll(s.token),
      ]);
      if (stations.length > 0) {
        selectedStation = stations[0].id;
        await loadStationData();
      }
      findings = await stationApi.getFindings(s.token, undefined);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function loadStationData() {
    const s = $session; if (!s || !selectedStation) return;
    try {
      [inventory, annotations, personnelLog, supplyRequests] = await Promise.all([
        stationApi.getInventory(s.token, selectedStation),
        stationApi.getAnnotations(s.token, selectedStation),
        stationApi.getPersonnelLog(s.token, selectedStation),
        stationApi.getStationSupplyRequests(s.token, selectedStation),
      ]);
    } catch {}
  }

  async function updateInventory() {
    const s = $session; if (!s || !selectedStation) return;
    if (!invCategory || !invItem) { showToast('Category and item required', 'error'); return; }
    try {
      await stationApi.updateInventory(s.token, selectedStation, invCategory, invItem, invQty, invUnit || undefined);
      showToast('Inventory updated', 'success');
      invOpen = false; invCategory = ''; invItem = ''; invQty = 0; invUnit = '';
      inventory = await stationApi.getInventory(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addAnnotation() {
    const s = $session; if (!s || !selectedStation) return;
    if (!annotSection) { showToast('Section name required', 'error'); return; }
    try {
      await stationApi.addAnnotation(s.token, selectedStation, annotSection, annotDesc || undefined);
      showToast('Annotation added', 'success');
      annotOpen = false; annotSection = ''; annotDesc = '';
      annotations = await stationApi.getAnnotations(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFindings() {
    const s = $session; if (!s || !selectedStation) return;
    if (!findTitle) { showToast('Title required', 'error'); return; }
    try {
      await stationApi.submitFindings(s.token, selectedStation, findTitle, findDesc || undefined, findPrivate);
      showToast('Findings submitted', 'success');
      findOpen = false; findTitle = ''; findDesc = ''; findPrivate = false;
      findings = await stationApi.getFindings(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function logPersonnelEvent() {
    const s = $session; if (!s || !selectedStation) return;
    if (!personnelUserSelected || !personnelEventType) { showToast('Personnel and event type required', 'error'); return; }
    try {
      await stationApi.logPersonnelEvent(s.token, selectedStation, personnelUserSelected.id, personnelEventType, personnelNotes || undefined);
      showToast('Personnel event logged', 'success');
      personnelOpen = false; personnelUserSelected = null; personnelEventType = ''; personnelNotes = '';
      personnelLog = await stationApi.getPersonnelLog(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitSupplyRequest() {
    const s = $session; if (!s || !selectedStation) return;
    if (!supplyTitle) { showToast('Title required', 'error'); return; }
    try {
      await stationApi.submitSupplyRequest(s.token, selectedStation, supplyTitle, supplyItems || undefined, supplyCost ? parseFloat(supplyCost) : undefined);
      showToast('Supply request submitted', 'success');
      supplyOpen = false; supplyTitle = ''; supplyItems = ''; supplyCost = '';
      await loadStationData();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openSupplyReview(req: any) {
    supplyReviewTarget = req; supplyReviewDecision = ''; supplyReviewNotes = ''; supplyReviewOpen = true;
  }

  async function submitSupplyReview() {
    const s = $session; if (!s || !supplyReviewTarget) return;
    if (!supplyReviewDecision) { showToast('Decision required', 'error'); return; }
    try {
      await stationApi.reviewSupplyRequest(s.token, supplyReviewTarget.id, supplyReviewDecision, supplyReviewNotes || undefined);
      showToast('Supply request reviewed', 'success');
      supplyReviewOpen = false;
      await loadStationData();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitAbandonment() {
    const s = $session; if (!s || !selectedStation) return;
    if (!abandonReason.trim()) { showToast('Reason required', 'error'); return; }
    try {
      await stationApi.submitAbandonment(s.token, selectedStation, abandonReason);
      showToast('Abandonment request submitted — a Director vote has been automatically initiated', 'success');
      abandonOpen = false; abandonReason = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const invCols = [
    { key: 'category', label: 'Category' },
    { key: 'item_name', label: 'Item' },
    { key: 'quantity', label: 'Qty' },
    { key: 'unit', label: 'Unit' },
  ];
  const annotCols = [
    { key: 'section_name', label: 'Section' },
    { key: 'description', label: 'Description' },
  ];
  const personnelCols = [
    { key: 'event_type', label: 'Event' },
    { key: 'user_id', label: 'User ID' },
    { key: 'notes', label: 'Notes' },
    { key: 'created_at', label: 'Time' },
  ];
  const findingsCols = [
    { key: 'title', label: 'Title' },
    { key: 'description', label: 'Description' },
    { key: 'is_private', label: 'Private' },
  ];
  const supplyCols = [
    { key: 'title', label: 'Title' },
    { key: 'items', label: 'Items' },
    { key: 'total_cost', label: 'Est. Cost' },
    { key: 'status', label: 'Status' },
  ];
  const supplyReviewOpts = [
    { value: 'approved', label: 'Approve' },
    { value: 'rejected', label: 'Reject' },
  ];
</script>

<svelte:head><title>RUSA IMS — Space Station</title></svelte:head>

<PageShell title="Space Station" subtitle="Inventory, map annotations, personnel log, findings, supply requests, and abandonment">
  {#if stations.length > 0}
    <div class="station-select">
      <label class="field-label" for="station-select">Station</label>
      <select id="station-select" class="field-input" bind:value={selectedStation} onchange={loadStationData}>
        {#each stations as st}
          <option value={st.id}>{st.name}</option>
        {/each}
      </select>
      <button class="btn-danger-outline" onclick={() => abandonOpen = true} style="margin-left: auto">⚠ Request Abandonment</button>
    </div>
  {/if}

  <div class="tabs">
    <button class="tab" class:active={activeTab==='inventory'} onclick={() => activeTab='inventory'}>Inventory</button>
    <button class="tab" class:active={activeTab==='annotations'} onclick={() => activeTab='annotations'}>Map Annotations</button>
    <button class="tab" class:active={activeTab==='personnel'} onclick={() => activeTab='personnel'}>Personnel Log</button>
    <button class="tab" class:active={activeTab==='findings'} onclick={() => activeTab='findings'}>Findings</button>
    <button class="tab" class:active={activeTab==='supply'} onclick={() => activeTab='supply'}>Supply Requests</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'inventory'}
    <div class="section-bar">
      <h2 class="section-title">Station Inventory</h2>
      <button class="btn-primary" onclick={() => invOpen = true}>+ Update Inventory</button>
    </div>
    <Table columns={invCols} rows={inventory} />
  {:else if activeTab === 'annotations'}
    <div class="section-bar">
      <h2 class="section-title">Map Annotations</h2>
      <button class="btn-primary" onclick={() => annotOpen = true}>+ Add Annotation</button>
    </div>
    <Table columns={annotCols} rows={annotations} />
  {:else if activeTab === 'personnel'}
    <div class="section-bar">
      <h2 class="section-title">Personnel Log</h2>
      <button class="btn-primary" onclick={() => personnelOpen = true}>+ Log Event</button>
    </div>
    <Table columns={personnelCols} rows={personnelLog} />
  {:else if activeTab === 'findings'}
    <div class="section-bar">
      <h2 class="section-title">Station Findings</h2>
      <button class="btn-primary" onclick={() => findOpen = true}>+ Submit Findings</button>
    </div>
    <Table columns={findingsCols} rows={findings} />
  {:else if activeTab === 'supply'}
    <div class="section-bar">
      <h2 class="section-title">Supply Requests</h2>
      <button class="btn-primary" onclick={() => supplyOpen = true}>+ Submit Request</button>
    </div>
    {#if supplyRequests.length === 0}
      <p class="empty">No supply requests for this station.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Items</th>
              <th>Est. Cost</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each supplyRequests as req}
              <tr>
                <td>{req.title}</td>
                <td>{req.items ?? '—'}</td>
                <td>{req.total_cost != null ? req.total_cost.toLocaleString() : '—'}</td>
                <td><span class="badge status-{req.status}">{req.status ?? '—'}</span></td>
                <td>
                  {#if isDirector && req.status === 'pending'}
                    <button class="btn-small" onclick={() => openSupplyReview(req)}>Review</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</PageShell>

<Modal bind:open={invOpen} title="Update Inventory">
  <div class="form">
    <Field label="Category" bind:value={invCategory} required />
    <Field label="Item Name" bind:value={invItem} required />
    <Field label="Quantity" type="number" bind:value={invQty} />
    <Field label="Unit" bind:value={invUnit} placeholder="e.g. kg, units, litres" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateInventory}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={annotOpen} title="Add Map Annotation">
  <div class="form">
    <Field label="Section Name" bind:value={annotSection} required />
    <Field label="Description" type="textarea" bind:value={annotDesc} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => annotOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addAnnotation}>Add</button>
    </div>
  </div>
</Modal>

<Modal bind:open={findOpen} title="Submit Findings">
  <div class="form">
    <Field label="Title" bind:value={findTitle} required />
    <Field label="Description" type="textarea" bind:value={findDesc} />
    <div class="field">
      <label class="field-label" style="display:flex;align-items:center;gap:0.5rem;cursor:pointer">
        <input type="checkbox" bind:checked={findPrivate} />
        Mark as Private (only visible to you)
      </label>
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => findOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitFindings}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={personnelOpen} title="Log Personnel Event">
  <div class="form">
    <div class="field">
      <label class="field-label">Personnel</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={personnelUserSelected} />
    </div>
    <Field label="Event Type" type="select" bind:value={personnelEventType} options={personnelEventOpts} required />
    <Field label="Notes" type="textarea" bind:value={personnelNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => personnelOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={logPersonnelEvent}>Log Event</button>
    </div>
  </div>
</Modal>

<Modal bind:open={supplyOpen} title="Submit Supply Request">
  <div class="form">
    <Field label="Title" bind:value={supplyTitle} required />
    <Field label="Items Requested" type="textarea" bind:value={supplyItems} hint="List all items and quantities" />
    <Field label="Estimated Total Cost" type="number" bind:value={supplyCost} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => supplyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitSupplyRequest}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={supplyReviewOpen} title="Review Supply Request">
  <div class="form">
    {#if supplyReviewTarget}
      <div class="info-block">
        <p class="info-text">Request: <strong>{supplyReviewTarget.title}</strong></p>
        {#if supplyReviewTarget.items}<p class="info-text">Items: {supplyReviewTarget.items}</p>{/if}
        {#if supplyReviewTarget.total_cost != null}<p class="info-text">Est. Cost: <strong>{supplyReviewTarget.total_cost.toLocaleString()}</strong></p>{/if}
      </div>
      <Field label="Decision" type="select" bind:value={supplyReviewDecision} options={supplyReviewOpts} required />
      <Field label="Notes (optional)" type="textarea" bind:value={supplyReviewNotes} />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => supplyReviewOpen = false}>Cancel</button>
        <button class="btn-primary" onclick={submitSupplyReview}>Submit Decision</button>
      </div>
    {/if}
  </div>
</Modal>

<Modal bind:open={abandonOpen} title="Request Station Abandonment">
  <div class="form">
    <div class="warning-block">
      ⚠ This will submit a formal request to abandon the selected station. Submitting this request will <strong>automatically initiate a Director vote</strong>. Abandonment requires the vote to pass (quorum: 8 of 13 Directors).
    </div>
    <p class="info-text">Station: <strong>{stations.find(s => s.id === selectedStation)?.name ?? '—'}</strong></p>
    <Field label="Reason for Abandonment" type="textarea" bind:value={abandonReason} rows={5} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => abandonOpen = false}>Cancel</button>
      <button class="btn-danger" onclick={submitAbandonment}>Submit Abandonment Request</button>
    </div>
  </div>
</Modal>

<style>
  .station-select { display: flex; align-items: center; gap: 1rem; margin-bottom: 1.5rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; white-space: nowrap; }
  .field-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.5rem 0.875rem; }
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.25rem 0.625rem; }
  .btn-small:hover { border-color: #3d7fff; color: #3d7fff; }
  .btn-danger { background: linear-gradient(135deg, #dc2626, #ef4444); border: none; border-radius: 4px; color: #fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-danger-outline { background: none; border: 1px solid #dc2626; border-radius: 4px; color: #ef4444; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.05em; padding: 0.4rem 0.875rem; }
  .btn-danger-outline:hover { background: rgba(220,38,38,0.1); }
  .table-wrap { overflow-x: auto; }
  .data-table { width: 100%; border-collapse: collapse; font-family: 'Outfit', sans-serif; font-size: 0.85rem; }
  .data-table th { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; padding: 0.5rem 0.75rem; text-align: left; text-transform: uppercase; border-bottom: 1px solid #1e2d4a; }
  .data-table td { border-bottom: 1px solid #0d1a2e; color: #c8d8f0; padding: 0.6rem 0.75rem; }
  .data-table tr:hover td { background: rgba(61,127,255,0.04); }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .status-pending { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .status-approved { background: rgba(0,200,83,0.12); color: #00c853; }
  .status-rejected { background: rgba(255,68,102,0.12); color: #ff4466; }
  .info-block { background: rgba(61,127,255,0.05); border: 1px solid #1e2d4a; border-radius: 4px; display: flex; flex-direction: column; gap: 0.25rem; padding: 0.75rem; }
  .warning-block { background: rgba(220,38,38,0.08); border: 1px solid rgba(220,38,38,0.3); border-radius: 4px; color: #fca5a5; font-size: 0.85rem; padding: 0.75rem; }
</style>
