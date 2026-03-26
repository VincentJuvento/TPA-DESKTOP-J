<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { sanitaryApi, governanceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'tasks' | 'nomad_tasks' | 'inventory' | 'disposal' | 'wastewater' | 'inspections';
  let activeTab = $state<Tab>('tasks');

  let tasks: any[] = $state([]);
  let nomadTasks: any[] = $state([]);
  let inventory: any[] = $state([]);
  let disposalLogs: any[] = $state([]);
  let wastewaterLogs: any[] = $state([]);
  let inspections: any[] = $state([]);
  let loading = $state(false);

  // Task form
  let taskOpen = $state(false);
  let tTitle = $state('');
  let tDesc = $state('');
  let tDivision = $state('');
  let tDue = $state('');

  let allUsers: any[] = $state([]);
  let assigneeSelected: any = $state(null);

  // Update nomad task status
  let nomadTaskStatusOpen = $state(false);
  let selectedNomadTask: any = $state(null);
  let nomadTaskStatus = $state('pending');

  // Update sanitary task status (for crew members)
  let sanitaryTaskStatusOpen = $state(false);
  let selectedSanitaryTask: any = $state(null);
  let sanitaryTaskStatus = $state('pending');

  // Inventory form
  let invOpen = $state(false);
  let invItem = $state('');
  let invCategory = $state('');
  let invQty = $state(0);
  let invUnit = $state('');

  // Disposal form
  let disposalOpen = $state(false);
  let dItem = $state('');
  let dQty = $state(0);
  let dUnit = $state('');
  let dMethod = $state('');
  let dHazard = $state('');
  let dNotes = $state('');

  // Wastewater form
  let wwOpen = $state(false);
  let wwVolume = $state(0);
  let wwUnit = $state('');
  let wwMethod = $state('');
  let wwPh = $state(7);

  // Inspection form
  let inspOpen = $state(false);
  let inspLocation = $state('');
  let inspDate = $state('');
  let inspFindings = $state('');
  let inspViolations = $state('');
  let inspRecommendations = $state('');

  const isHead = $derived(canPerform($session, 'head_of_sanitary'));
  const isDisposalCrew = $derived(canPerform($session, 'disposal_crew'));
  const isWastewaterCrew = $derived(canPerform($session, 'wastewater_crew'));
  const isInspector = $derived(canPerform($session, 'sanitary_inspector'));

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [tasks, nomadTasks, inventory, disposalLogs, wastewaterLogs, inspections] = await Promise.all([
        sanitaryApi.getTasks(s.token),
        governanceApi.getNomadTasks(s.token),
        sanitaryApi.getInventory(s.token),
        sanitaryApi.getDisposalLogs(s.token),
        sanitaryApi.getWastewaterLogs(s.token),
        sanitaryApi.getInspections(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !tTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await sanitaryApi.assignTask(s.token, assigneeSelected.id, tTitle, tDesc || undefined, tDivision || undefined, tDue ? tDue + 'T00:00:00Z' : undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; assigneeSelected = null; tTitle = ''; tDesc = ''; tDivision = ''; tDue = '';
      tasks = await sanitaryApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openNomadTaskStatus(task: any) {
    selectedNomadTask = task;
    nomadTaskStatus = task.status ?? 'pending';
    nomadTaskStatusOpen = true;
  }

  async function updateNomadTaskStatus() {
    const s = $session; if (!s || !selectedNomadTask) return;
    try {
      await governanceApi.updateNomadTaskStatus(s.token, selectedNomadTask.id, nomadTaskStatus);
      showToast('Task status updated', 'success');
      nomadTaskStatusOpen = false;
      nomadTasks = await governanceApi.getNomadTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openSanitaryTaskStatus(task: any) {
    selectedSanitaryTask = task;
    sanitaryTaskStatus = task.status ?? 'pending';
    sanitaryTaskStatusOpen = true;
  }

  async function updateSanitaryTaskStatus() {
    const s = $session; if (!s || !selectedSanitaryTask) return;
    try {
      await sanitaryApi.updateTask(s.token, selectedSanitaryTask.id, sanitaryTaskStatus);
      showToast('Task status updated', 'success');
      sanitaryTaskStatusOpen = false;
      tasks = await sanitaryApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function updateInventory() {
    const s = $session; if (!s) return;
    if (!invItem) { showToast('Item name required', 'error'); return; }
    try {
      await sanitaryApi.updateInventory(s.token, invItem, invCategory || undefined, invQty || undefined, invUnit || undefined);
      showToast('Inventory updated', 'success');
      invOpen = false; invItem = ''; invCategory = ''; invQty = 0; invUnit = '';
      inventory = await sanitaryApi.getInventory(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addDisposal() {
    const s = $session; if (!s) return;
    if (!dItem || !dQty) { showToast('Item and quantity required', 'error'); return; }
    try {
      await sanitaryApi.addDisposalLog(s.token, dItem, dQty, dUnit || undefined, dMethod || undefined, dHazard || undefined, dNotes || undefined);
      showToast('Disposal logged', 'success');
      disposalOpen = false; dItem = ''; dQty = 0; dUnit = ''; dMethod = ''; dHazard = ''; dNotes = '';
      disposalLogs = await sanitaryApi.getDisposalLogs(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addWastewater() {
    const s = $session; if (!s) return;
    if (!wwVolume) { showToast('Volume required', 'error'); return; }
    try {
      await sanitaryApi.addWastewaterLog(s.token, wwVolume, wwUnit || undefined, wwMethod || undefined, wwPh || undefined);
      showToast('Wastewater log added', 'success');
      wwOpen = false; wwVolume = 0; wwUnit = ''; wwMethod = ''; wwPh = 7;
      wastewaterLogs = await sanitaryApi.getWastewaterLogs(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createInspection() {
    const s = $session; if (!s) return;
    if (!inspLocation || !inspDate || !inspFindings) { showToast('Location, date, and findings required', 'error'); return; }
    try {
      await sanitaryApi.createInspection(s.token, inspLocation, inspDate + 'T00:00:00Z', inspFindings, inspViolations || undefined, inspRecommendations || undefined);
      showToast('Inspection report created', 'success');
      inspOpen = false; inspLocation = ''; inspDate = ''; inspFindings = ''; inspViolations = ''; inspRecommendations = '';
      inspections = await sanitaryApi.getInspections(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const taskCols = [
    { key: 'title', label: 'Task' },
    { key: 'assigned_to_name', label: 'Assigned To' },
    { key: 'division', label: 'Division' },
    { key: 'status', label: 'Status' },
  ];
  const invCols = [
    { key: 'item_name', label: 'Item' },
    { key: 'category', label: 'Category' },
    { key: 'quantity', label: 'Qty' },
    { key: 'unit', label: 'Unit' },
  ];
  const disposalCols = [
    { key: 'item_name', label: 'Item' },
    { key: 'quantity', label: 'Qty' },
    { key: 'disposal_method', label: 'Method' },
    { key: 'hazard_level', label: 'Hazard' },
  ];
  const wwCols = [
    { key: 'volume_treated', label: 'Volume' },
    { key: 'treatment_method', label: 'Method' },
    { key: 'ph_level', label: 'pH' },
    { key: 'logged_at', label: 'Date' },
  ];
  const inspCols = [
    { key: 'location', label: 'Location' },
    { key: 'inspection_date', label: 'Date' },
    { key: 'findings', label: 'Findings' },
  ];
  const hazardOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'extreme', label: 'Extreme' },
  ];
</script>

<svelte:head><title>RUSA IMS — Sanitary & Waste</title></svelte:head>

<PageShell title="Sanitary & Waste Management" subtitle="Tasks, inventory, disposal logs, wastewater, and inspections">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>Tasks</button>
    {#if isHead}
      <button class="tab" class:active={activeTab==='nomad_tasks'} onclick={() => activeTab='nomad_tasks'}>My Nomad Tasks</button>
    {/if}
    <button class="tab" class:active={activeTab==='inventory'} onclick={() => activeTab='inventory'}>Inventory</button>
    <button class="tab" class:active={activeTab==='disposal'} onclick={() => activeTab='disposal'}>Disposal</button>
    <button class="tab" class:active={activeTab==='wastewater'} onclick={() => activeTab='wastewater'}>Wastewater</button>
    <button class="tab" class:active={activeTab==='inspections'} onclick={() => activeTab='inspections'}>Inspections</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">{isHead ? 'All Sanitary Tasks' : 'My Assigned Tasks'}</h2>
      {#if isHead}
        <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if tasks.length === 0}
      <p class="empty">{isHead ? 'No tasks yet.' : 'No tasks have been assigned to you.'}</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Division</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each tasks as task}
              <tr>
                <td>{task.title}</td>
                <td>{task.division ?? '—'}</td>
                <td><span class="badge {task.status === 'completed' ? 'badge-done' : task.status === 'in_progress' ? 'badge-progress' : 'badge-open'}">{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>
                  {#if !isHead}
                    <button class="btn-small" onclick={() => openSanitaryTaskStatus(task)}>Update Status</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'nomad_tasks'}
    <div class="section-bar">
      <h2 class="section-title">Tasks Assigned by The Nomad</h2>
    </div>
    {#if nomadTasks.length === 0}
      <p class="empty">No tasks have been assigned to you by The Nomad.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each nomadTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class="badge {task.status === 'completed' ? 'badge-done' : task.status === 'in_progress' ? 'badge-progress' : 'badge-open'}">{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>
                  <button class="btn-small" onclick={() => openNomadTaskStatus(task)}>Update Status</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'inventory'}
    <div class="section-bar">
      <h2 class="section-title">Sanitary Inventory</h2>
      {#if isHead}
        <button class="btn-primary" onclick={() => invOpen = true}>+ Update Inventory</button>
      {/if}
    </div>
    <Table columns={invCols} rows={inventory} />
  {:else if activeTab === 'disposal'}
    <div class="section-bar">
      <h2 class="section-title">Disposal Logs</h2>
      {#if isDisposalCrew}
        <button class="btn-primary" onclick={() => disposalOpen = true}>+ Log Disposal</button>
      {/if}
    </div>
    <Table columns={disposalCols} rows={disposalLogs} />
  {:else if activeTab === 'wastewater'}
    <div class="section-bar">
      <h2 class="section-title">Wastewater Treatment Logs</h2>
      {#if isWastewaterCrew}
        <button class="btn-primary" onclick={() => wwOpen = true}>+ Add Log</button>
      {/if}
    </div>
    <Table columns={wwCols} rows={wastewaterLogs} />
  {:else if activeTab === 'inspections'}
    <div class="section-bar">
      <h2 class="section-title">Inspection Reports</h2>
      {#if isInspector}
        <button class="btn-primary" onclick={() => inspOpen = true}>+ Create Report</button>
      {/if}
    </div>
    <Table columns={inspCols} rows={inspections} />
  {/if}
</PageShell>

<Modal bind:open={taskOpen} title="Assign Sanitary Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assigneeSelected} />
    </div>
    <Field label="Title" bind:value={tTitle} required />
    <Field label="Description" type="textarea" bind:value={tDesc} />
    <Field label="Division" bind:value={tDivision} placeholder="e.g. cleanup, disposal" />
    <Field label="Due Date" type="date" bind:value={tDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { taskOpen = false; assigneeSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={nomadTaskStatusOpen} title="Update Nomad Task Status">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedNomadTask?.title}</strong></p>
    <Field label="Status" type="select" bind:value={nomadTaskStatus} options={[
      { value: 'pending', label: 'Pending' },
      { value: 'in_progress', label: 'In Progress' },
      { value: 'completed', label: 'Completed' },
      { value: 'cancelled', label: 'Cancelled' },
    ]} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => nomadTaskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateNomadTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={sanitaryTaskStatusOpen} title="Update Task Status">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedSanitaryTask?.title}</strong></p>
    <Field label="Status" type="select" bind:value={sanitaryTaskStatus} options={[
      { value: 'pending', label: 'Pending' },
      { value: 'in_progress', label: 'In Progress' },
      { value: 'completed', label: 'Completed' },
      { value: 'cancelled', label: 'Cancelled' },
    ]} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => sanitaryTaskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateSanitaryTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={invOpen} title="Update Sanitary Inventory">
  <div class="form">
    <Field label="Item Name" bind:value={invItem} required />
    <Field label="Category" bind:value={invCategory} />
    <Field label="Quantity" type="number" bind:value={invQty} />
    <Field label="Unit" bind:value={invUnit} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateInventory}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={disposalOpen} title="Log Disposal">
  <div class="form">
    <Field label="Item Name" bind:value={dItem} required />
    <Field label="Quantity" type="number" bind:value={dQty} required />
    <Field label="Unit" bind:value={dUnit} />
    <Field label="Disposal Method" bind:value={dMethod} placeholder="e.g. incineration, recycling" />
    <Field label="Hazard Level" type="select" bind:value={dHazard} options={hazardOpts} />
    <Field label="Notes" type="textarea" bind:value={dNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => disposalOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addDisposal}>Log</button>
    </div>
  </div>
</Modal>

<Modal bind:open={wwOpen} title="Log Wastewater Treatment">
  <div class="form">
    <Field label="Volume Treated" type="number" bind:value={wwVolume} required />
    <Field label="Unit" bind:value={wwUnit} placeholder="e.g. litres, m³" />
    <Field label="Treatment Method" bind:value={wwMethod} />
    <Field label="pH Level" type="number" bind:value={wwPh} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => wwOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addWastewater}>Log</button>
    </div>
  </div>
</Modal>

<Modal bind:open={inspOpen} title="Create Inspection Report">
  <div class="form">
    <Field label="Location" bind:value={inspLocation} required />
    <Field label="Inspection Date" type="date" bind:value={inspDate} required />
    <Field label="Findings" type="textarea" bind:value={inspFindings} required rows={4} />
    <Field label="Violations" type="textarea" bind:value={inspViolations} />
    <Field label="Recommendations" type="textarea" bind:value={inspRecommendations} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => inspOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createInspection}>Create</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.25rem 0.625rem; }
  .btn-small:hover { border-color: #3d7fff; color: #3d7fff; }
  .table-wrap { overflow-x: auto; }
  .data-table { width: 100%; border-collapse: collapse; font-family: 'Outfit', sans-serif; font-size: 0.85rem; }
  .data-table th { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; padding: 0.5rem 0.75rem; text-align: left; text-transform: uppercase; border-bottom: 1px solid #1e2d4a; }
  .data-table td { border-bottom: 1px solid #0d1a2e; color: #c8d8f0; padding: 0.6rem 0.75rem; }
  .data-table tr:hover td { background: rgba(61,127,255,0.04); }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
</style>
