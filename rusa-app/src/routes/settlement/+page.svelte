<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { settlementApi, governanceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'tasks' | 'nomad_tasks' | 'supply' | 'anomalies' | 'farm_reports' | 'troublesome_reports' | 'civil_reports';
  let activeTab = $state<Tab>('tasks');

  let tasks: any[] = $state([]);
  let nomadTasks: any[] = $state([]);
  let supplyReqs: any[] = $state([]);
  let anomalies: any[] = $state([]);
  let farmReports: any[] = $state([]);
  let troublesomeReports: any[] = $state([]);
  let civilReports: any[] = $state([]);
  let loading = $state(false);

  // Task form
  let taskOpen = $state(false);
  let tTitle = $state('');
  let tDesc = $state('');
  let tDue = $state('');

  let allUsers: any[] = $state([]);
  let assigneeSelected: any = $state(null);

  // Update nomad task status
  let nomadTaskStatusOpen = $state(false);
  let selectedNomadTask: any = $state(null);
  let nomadTaskStatus = $state('pending');

  // Supply request
  let supplyOpen = $state(false);
  let sTitle = $state('');
  let sDesc = $state('');
  let sItems = $state('');

  // Anomaly form
  let anomalyOpen = $state(false);
  let aTitle = $state('');
  let aDesc = $state('');
  let aSeverity = $state('');

  // Farm report form
  let farmOpen = $state(false);
  let fTitle = $state('');
  let fContent = $state('');
  let fCropStatus = $state('');
  let fHealthNotes = $state('');

  // Troublesome settler report form
  let troublesomeOpen = $state(false);
  let trSettlerSelected: any = $state(null);
  let trDesc = $state('');

  // Civil engineer report form
  let civilOpen = $state(false);
  let ciTitle = $state('');
  let ciContent = $state('');
  let ciMaterials = $state('');
  let ciProgress = $state<number | undefined>(undefined);
  let ciProblems = $state('');
  let ciPlans = $state('');

  const isCommander = $derived(canPerform($session, 'settler_commander'));
  const isCivilEngineer = $derived(canPerform($session, 'civil_engineer'));

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [tasks, nomadTasks, supplyReqs, anomalies, farmReports, troublesomeReports, civilReports] = await Promise.all([
        settlementApi.getTasks(s.token),
        governanceApi.getNomadTasks(s.token),
        settlementApi.getSupplyRequests(s.token),
        settlementApi.getAnomalyReports(s.token),
        settlementApi.getFarmReports(s.token),
        settlementApi.getTroublesomeReports(s.token),
        settlementApi.getCivilEngineerReports(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !tTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await settlementApi.assignTask(s.token, assigneeSelected.id, tTitle, tDesc || undefined, tDue ? tDue + 'T00:00:00Z' : undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; assigneeSelected = null; tTitle = ''; tDesc = ''; tDue = '';
      tasks = await settlementApi.getTasks(s.token);
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

  async function submitSupply() {
    const s = $session; if (!s) return;
    if (!sTitle) { showToast('Title required', 'error'); return; }
    try {
      await settlementApi.submitSupplyRequest(s.token, undefined, sTitle, sDesc || undefined, sItems || undefined);
      showToast('Supply request submitted', 'success');
      supplyOpen = false; sTitle = ''; sDesc = ''; sItems = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitAnomaly() {
    const s = $session; if (!s) return;
    if (!aTitle) { showToast('Title required', 'error'); return; }
    try {
      await settlementApi.submitAnomalyReport(s.token, undefined, aTitle, aDesc || undefined, aSeverity || undefined);
      showToast('Anomaly reported', 'success');
      anomalyOpen = false; aTitle = ''; aDesc = ''; aSeverity = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFarmReport() {
    const s = $session; if (!s) return;
    if (!fTitle || !fContent) { showToast('Title and content required', 'error'); return; }
    try {
      await settlementApi.submitFarmReport(s.token, undefined, fTitle, fContent, fCropStatus || undefined, fHealthNotes || undefined);
      showToast('Farm report submitted', 'success');
      farmOpen = false; fTitle = ''; fContent = ''; fCropStatus = ''; fHealthNotes = '';
      farmReports = await settlementApi.getFarmReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitTroublesomeReport() {
    const s = $session; if (!s) return;
    if (!trSettlerSelected || !trDesc) { showToast('Settler and description required', 'error'); return; }
    try {
      await settlementApi.submitTroublesomeReport(s.token, trSettlerSelected.id, trDesc);
      showToast('Troublesome settler report submitted', 'success');
      troublesomeOpen = false; trSettlerSelected = null; trDesc = '';
      troublesomeReports = await settlementApi.getTroublesomeReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitCivilReport() {
    const s = $session; if (!s) return;
    if (!ciTitle || !ciContent) { showToast('Title and content required', 'error'); return; }
    let materialsJson: string | undefined;
    if (ciMaterials.trim()) {
      const lines = ciMaterials.split('\n').map(l => l.trim()).filter(Boolean);
      materialsJson = JSON.stringify(lines);
    }
    try {
      await settlementApi.submitCivilEngineerReport(s.token, ciTitle, ciContent, undefined, undefined, materialsJson, ciProgress, ciProblems || undefined, ciPlans || undefined);
      showToast('Civil engineer report submitted', 'success');
      civilOpen = false; ciTitle = ''; ciContent = ''; ciMaterials = ''; ciProgress = undefined; ciProblems = ''; ciPlans = '';
      civilReports = await settlementApi.getCivilEngineerReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function statusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'pending': return 'badge badge-open';
      case 'in_progress': return 'badge badge-progress';
      case 'completed': return 'badge badge-done';
      case 'cancelled': return 'badge badge-cancelled';
      default: return 'badge';
    }
  }

  const taskCols = [
    { key: 'title', label: 'Task' },
    { key: 'assigned_to_name', label: 'Assigned To' },
    { key: 'status', label: 'Status' },
    { key: 'due_date', label: 'Due' },
  ];
  const nomadTaskStatusOpts = [
    { value: 'pending', label: 'Pending' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
  const severityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
</script>

<svelte:head><title>RUSA IMS — Settlement Operations</title></svelte:head>

<PageShell title="Settlement Operations" subtitle="Task management, supply requests, anomaly reports, and disciplinary actions">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>Tasks</button>
    {#if isCommander}
      <button class="tab" class:active={activeTab==='nomad_tasks'} onclick={() => activeTab='nomad_tasks'}>My Nomad Tasks</button>
    {/if}
    <button class="tab" class:active={activeTab==='supply'} onclick={() => activeTab='supply'}>Supply Requests</button>
    <button class="tab" class:active={activeTab==='anomalies'} onclick={() => activeTab='anomalies'}>Anomaly Reports</button>
    {#if canPerform($session, 'farmer') || canPerform($session, 'settler_commander')}
      <button class="tab" class:active={activeTab==='farm_reports'} onclick={() => activeTab='farm_reports'}>Farm Reports</button>
    {/if}
    <button class="tab" class:active={activeTab==='troublesome_reports'} onclick={() => activeTab='troublesome_reports'}>Troublesome Settlers</button>
    {#if isCivilEngineer || isCommander}
      <button class="tab" class:active={activeTab==='civil_reports'} onclick={() => activeTab='civil_reports'}>Civil Engineer Reports</button>
    {/if}
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">Settler Tasks</h2>
      <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
    </div>
    <Table columns={taskCols} rows={tasks} />
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
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
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
  {:else if activeTab === 'supply'}
    <div class="section-bar">
      <h2 class="section-title">Supply Requests</h2>
      <button class="btn-primary" onclick={() => supplyOpen = true}>+ New Request</button>
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'status',label:'Status'},{key:'items',label:'Items'}]} rows={supplyReqs} />
  {:else if activeTab === 'anomalies'}
    <div class="section-bar">
      <h2 class="section-title">Anomaly Reports</h2>
      <button class="btn-primary" onclick={() => anomalyOpen = true}>+ Report Anomaly</button>
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'severity',label:'Severity'},{key:'status',label:'Status'}]} rows={anomalies} />
  {:else if activeTab === 'farm_reports'}
    <div class="section-bar">
      <h2 class="section-title">Farm Reports</h2>
      {#if canPerform($session, 'farmer') || canPerform($session, 'settler_commander')}
        <button class="btn-primary" onclick={() => farmOpen = true}>+ Submit Farm Report</button>
      {/if}
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'crop_status',label:'Crop Status'},{key:'created_at',label:'Created At'}]} rows={farmReports} />
  {:else if activeTab === 'troublesome_reports'}
    <div class="section-bar">
      <h2 class="section-title">Troublesome Settler Reports</h2>
      <button class="btn-primary" onclick={() => troublesomeOpen = true}>+ Report Troublesome Settler</button>
    </div>
    {#if isCommander}
      <p class="info-hint">As Settler Commander, you can view all reports and issue house arrests or deportation requests as follow-up actions.</p>
    {/if}
    <Table columns={[{key:'description',label:'Description'},{key:'status',label:'Status'},{key:'created_at',label:'Reported At'}]} rows={troublesomeReports} />
  {:else if activeTab === 'civil_reports'}
    <div class="section-bar">
      <h2 class="section-title">Civil Engineer Progress Reports</h2>
      {#if isCivilEngineer || isCommander}
        <button class="btn-primary" onclick={() => civilOpen = true}>+ Submit Progress Report</button>
      {/if}
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'progress_percentage',label:'Progress %'},{key:'created_at',label:'Submitted At'}]} rows={civilReports} />
  {/if}
</PageShell>

<Modal bind:open={taskOpen} title="Assign Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assigneeSelected} />
    </div>
    <Field label="Title" bind:value={tTitle} required />
    <Field label="Description" type="textarea" bind:value={tDesc} />
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
    <Field label="Status" type="select" bind:value={nomadTaskStatus} options={nomadTaskStatusOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => nomadTaskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateNomadTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={supplyOpen} title="Submit Supply Request">
  <div class="form">
    <Field label="Title" bind:value={sTitle} required />
    <Field label="Description" type="textarea" bind:value={sDesc} />
    <Field label="Items" type="textarea" bind:value={sItems} hint="List items, one per line" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => supplyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitSupply}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={anomalyOpen} title="Report Anomaly">
  <div class="form">
    <Field label="Title" bind:value={aTitle} required />
    <Field label="Description" type="textarea" bind:value={aDesc} />
    <Field label="Severity" type="select" bind:value={aSeverity} options={severityOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => anomalyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitAnomaly}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={farmOpen} title="Submit Farm Report">
  <div class="form">
    <Field label="Title" bind:value={fTitle} required />
    <Field label="Content" type="textarea" bind:value={fContent} required />
    <Field label="Crop Status" bind:value={fCropStatus} />
    <Field label="Health Check Notes" type="textarea" bind:value={fHealthNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => farmOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitFarmReport}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={troublesomeOpen} title="Report Troublesome Settler">
  <div class="form">
    <p class="info-text">Report a settler who is causing trouble to the Settler Commander. The Commander may place them on house arrest and request deportation.</p>
    <div class="field">
      <label class="field-label">Settler to Report</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={trSettlerSelected} placeholder="Search settler by name or username…" />
    </div>
    <Field label="Description" type="textarea" bind:value={trDesc} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { troublesomeOpen = false; trSettlerSelected = null; trDesc = ''; }}>Cancel</button>
      <button class="btn-primary" onclick={submitTroublesomeReport}>Submit Report</button>
    </div>
  </div>
</Modal>

<Modal bind:open={civilOpen} title="Civil Engineer Progress Report">
  <div class="form">
    <Field label="Title" bind:value={ciTitle} required />
    <Field label="Description of Work Done" type="textarea" bind:value={ciContent} required />
    <Field label="Materials Used" type="textarea" bind:value={ciMaterials} hint="List each material on a new line (e.g. Steel beams, Concrete, Glass panels)" />
    <div class="field">
      <label class="field-label">Progress Percentage</label>
      <input class="field-input" type="number" min="0" max="100" bind:value={ciProgress} placeholder="0–100" />
    </div>
    <Field label="Problems Encountered" type="textarea" bind:value={ciProblems} />
    <Field label="Plans for Next Steps" type="textarea" bind:value={ciPlans} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => civilOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitCivilReport}>Submit</button>
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
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .info-hint { font-size: 0.8rem; color: #4a5d82; font-style: italic; margin-bottom: 0.75rem; }
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
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-cancelled { background: rgba(255,68,102,0.12); color: #ff4466; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .field-input { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.5rem 0.75rem; width: 100%; box-sizing: border-box; }
  .field-input:focus { border-color: #3d7fff; outline: none; }
</style>
