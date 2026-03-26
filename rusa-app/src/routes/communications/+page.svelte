<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { securityApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  let broadcasts: any[] = $state([]);
  let loading = $state(false);

  // Review modal
  let reviewOpen = $state(false);
  let selectedBroadcast: any = $state(null);
  let reviewStatus = $state('');
  let reviewNotes = $state('');
  let reviewScheduledAt = $state('');

  // Direct broadcast modal
  let directOpen = $state(false);
  let directTitle = $state('');
  let directBody = $state('');
  let directScheduledAt = $state('');
  let directPersonnel = $state<string[]>([]);
  let directDepartments = $state<string[]>([]);
  let directLocations = $state<string[]>([]);

  const role = $derived($session?.role_name ?? '');
  const canReview = $derived(role === 'the_guardian' || role === 'the_anchorman' || role === 'the_administrator');
  const canDirect = $derived(role === 'the_guardian' || role === 'the_anchorman' || role === 'the_administrator');
  const isOverseer = $derived(role === 'the_overseer');

  function addFilter(type: 'personnel' | 'departments' | 'locations') {
    if (type === 'personnel') directPersonnel = [...directPersonnel, ''];
    if (type === 'departments') directDepartments = [...directDepartments, ''];
    if (type === 'locations') directLocations = [...directLocations, ''];
  }
  function removeFilter(type: 'personnel' | 'departments' | 'locations', i: number) {
    if (type === 'personnel') directPersonnel = directPersonnel.filter((_, idx) => idx !== i);
    if (type === 'departments') directDepartments = directDepartments.filter((_, idx) => idx !== i);
    if (type === 'locations') directLocations = directLocations.filter((_, idx) => idx !== i);
  }

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      broadcasts = await securityApi.getBroadcastRequests(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  function openReview(item: any) {
    selectedBroadcast = item; reviewStatus = ''; reviewNotes = ''; reviewScheduledAt = ''; reviewOpen = true;
  }

  async function submitReview() {
    const s = $session; if (!s || !selectedBroadcast) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      const sched = reviewScheduledAt ? new Date(reviewScheduledAt).toISOString() : undefined;
      await securityApi.reviewBroadcast(s.token, selectedBroadcast.id, reviewStatus, reviewNotes || undefined, sched);
      showToast('Broadcast reviewed', 'success'); reviewOpen = false;
      broadcasts = await securityApi.getBroadcastRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function sendDirect() {
    const s = $session; if (!s) return;
    if (!directTitle.trim() || !directBody.trim()) { showToast('Subject and body required', 'error'); return; }

    const filters = {
      personnel: directPersonnel.filter(v => v.trim() !== ''),
      departments: directDepartments.filter(v => v.trim() !== ''),
      locations: directLocations.filter(v => v.trim() !== ''),
    };
    const hasFilters = filters.personnel.length > 0 || filters.departments.length > 0 || filters.locations.length > 0;
    const sched = directScheduledAt ? new Date(directScheduledAt).toISOString() : undefined;

    try {
      await securityApi.sendBroadcastDirect(s.token, directTitle, directBody, hasFilters ? filters : undefined, sched);
      showToast('Broadcast dispatched', 'success');
      directOpen = false;
      directTitle = ''; directBody = ''; directScheduledAt = '';
      directPersonnel = []; directDepartments = []; directLocations = [];
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  const broadcastCols = [
    { key: 'title', label: 'Title' },
    { key: 'requester_name', label: 'Requested By' },
    { key: 'target_audience', label: 'Audience' },
    { key: 'status', label: 'Status' },
    { key: 'created_at', label: 'Submitted' },
  ];
  const statusOpts = [
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
    { value: 'pending', label: 'Pending' },
  ];
</script>

<svelte:head><title>RUSA IMS — Communications</title></svelte:head>

<PageShell title="Communications" subtitle="Broadcast requests and system broadcasts">
  <div class="section-bar">
    <h2 class="section-title">Broadcast Requests</h2>
    <div class="header-info">{canReview ? 'Click a request to review it' : isOverseer ? 'Read-only access' : ''}</div>
  </div>
  {#if canDirect}
    <div class="actions-bar">
      <button class="btn-primary" onclick={() => directOpen = true}>+ Compose System Broadcast</button>
    </div>
  {/if}
  {#if loading}
    <p class="loading">Loading...</p>
  {:else}
    <Table columns={broadcastCols} rows={broadcasts} onRowClick={openReview} />
  {/if}
</PageShell>

<Modal bind:open={reviewOpen} title="Review Broadcast: {selectedBroadcast?.title}">
  <div class="form">
    {#if selectedBroadcast}
      <div class="preview-box">
        <p class="preview-label">Content</p>
        <p class="preview-text">{selectedBroadcast.content}</p>
      </div>
      <p class="meta-text">Audience: <strong>{selectedBroadcast.target_audience || 'All'}</strong></p>
    {/if}
    {#if canReview}
      <Field label="Decision" type="select" bind:value={reviewStatus} options={statusOpts} required />
      <Field label="Notes" type="textarea" bind:value={reviewNotes} />
      <Field label="Schedule Dispatch (Optional)" type="datetime-local" bind:value={reviewScheduledAt} />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
        <button class="btn-primary" onclick={submitReview}>Submit Decision</button>
      </div>
    {:else}
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => reviewOpen = false}>Close</button>
      </div>
    {/if}
  </div>
</Modal>

<Modal bind:open={directOpen} title="Compose System Broadcast">
  <div class="form">
    <Field label="Subject" bind:value={directTitle} required />
    <Field label="Message Body" type="textarea" bind:value={directBody} required rows={6} />
    <Field label="Schedule Dispatch (Optional)" type="datetime-local" bind:value={directScheduledAt} />

    <div class="filters">
      <div class="filters-title">Target Filters</div>

      <div class="filter-group">
        <div class="filter-head">
          <span>Specific Personnel (User IDs)</span>
          <button class="btn-small" onclick={() => addFilter('personnel')}>+ Add</button>
        </div>
        {#each directPersonnel as _, i}
          <div class="filter-row">
            <input class="field-input" bind:value={directPersonnel[i]} placeholder="User ID" />
            <button class="btn-small btn-danger" onclick={() => removeFilter('personnel', i)}>X</button>
          </div>
        {/each}
      </div>

      <div class="filter-group">
        <div class="filter-head">
          <span>Groups / Departments</span>
          <button class="btn-small" onclick={() => addFilter('departments')}>+ Add</button>
        </div>
        {#each directDepartments as _, i}
          <div class="filter-row">
            <input class="field-input" bind:value={directDepartments[i]} placeholder="Department name" />
            <button class="btn-small btn-danger" onclick={() => removeFilter('departments', i)}>X</button>
          </div>
        {/each}
      </div>

      <div class="filter-group">
        <div class="filter-head">
          <span>Planets / Sectors</span>
          <button class="btn-small" onclick={() => addFilter('locations')}>+ Add</button>
        </div>
        {#each directLocations as _, i}
          <div class="filter-row">
            <input class="field-input" bind:value={directLocations[i]} placeholder="Planet/Sector name" />
            <button class="btn-small btn-danger" onclick={() => removeFilter('locations', i)}>X</button>
          </div>
        {/each}
      </div>
    </div>

    <div class="form-actions">
      <button class="btn-secondary" onclick={() => directOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={sendDirect}>Dispatch</button>
    </div>
  </div>
</Modal>

<style>
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .header-info { font-size: 0.8rem; color: #4a5d82; }
  .actions-bar { display: flex; justify-content: flex-end; margin-bottom: 1rem; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .preview-box { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; padding: 1rem; }
  .preview-label { font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.08em; color: #4a5d82; margin-bottom: 0.5rem; }
  .preview-text { color: #e8eeff; font-size: 0.9rem; line-height: 1.6; }
  .meta-text { font-size: 0.82rem; color: #8fa3cc; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.3rem 0.6rem; }
  .btn-danger { background: #3a1a1a; color: #ff6b6b; border: 1px solid #ff6b6b; }

  .filters { margin-top: 0.5rem; padding: 1rem; background: #0f121b; border: 1px solid #1a2035; border-radius: 4px; }
  .filters-title { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #4a5d82; margin-bottom: 1rem; }
  .filter-group { margin-bottom: 1rem; }
  .filter-head { display: flex; justify-content: space-between; align-items: center; font-size: 0.85rem; color: #8b9bb4; margin-bottom: 0.5rem; }
  .filter-row { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; }
  .filter-row .field-input { flex: 1; background: #05070f; border: 1px solid #1a2035; color: #e2e8f0; padding: 0.5rem; border-radius: 4px; }
</style>
