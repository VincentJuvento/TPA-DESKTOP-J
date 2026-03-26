<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { medicalApi, userApi, budgetApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'shifts' | 'tasks' | 'inventory' | 'specializations' | 'budget';
  let activeTab = $state<Tab>('shifts');

  let shifts: any[] = $state([]);
  let tasks: any[] = $state([]);
  let inventory: any[] = $state([]);
  let specializations: any[] = $state([]);
  let loading = $state(false);

  // Shift form
  let shiftOpen = $state(false);
  let shiftStaffSelected: any = $state(null);
  let shiftStart = $state('');
  let shiftEnd = $state('');
  let shiftNotes = $state('');

  // Assign task form
  let taskOpen = $state(false);
  let tTitle = $state('');
  let tDesc = $state('');
  let tDue = $state('');

  let allUsers: any[] = $state([]);
  let assigneeSelected: any = $state(null);

  // Update task status form
  let taskStatusOpen = $state(false);
  let selectedTask: any = $state(null);
  let taskStatus = $state('pending');
  let taskProgressNotes = $state('');

  // Inventory form
  let invOpen = $state(false);
  let invItem = $state('');
  let invCategory = $state('');
  let invQty = $state(0);
  let invUnit = $state('');
  let invExpiry = $state('');

  // Specialization form
  let specOpen = $state(false);
  let specStaffSelected: any = $state(null);
  let specName = $state('');
  let specDate = $state('');

  // Budget form
  let budgetOpen = $state(false);
  let budgetTitle = $state('');
  let budgetDesc = $state('');
  let budgetAmount = $state(0);
  let budgetItems = $state('');

  const isHead = $derived(canPerform($session, 'head_of_medicine'));



  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [shifts, tasks, inventory, specializations] = await Promise.all([
        medicalApi.getShifts(s.token),
        medicalApi.getTasks(s.token),
        medicalApi.getInventory(s.token),
        medicalApi.getSpecializations(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function allocateShift() {
    const s = $session; if (!s) return;
    if (!shiftStaffSelected || !shiftStart || !shiftEnd) { showToast('Staff User, start, and end required', 'error'); return; }
    try {
      await medicalApi.allocateShift(s.token, shiftStaffSelected.id, shiftStart, shiftEnd, shiftNotes || undefined);
      showToast('Shift allocated', 'success');
      shiftOpen = false; shiftStaffSelected = null; shiftStart = ''; shiftEnd = ''; shiftNotes = '';
      shifts = await medicalApi.getShifts(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !tTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await medicalApi.assignTask(s.token, assigneeSelected.id, tTitle, tDesc || undefined, tDue || undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; assigneeSelected = null; tTitle = ''; tDesc = ''; tDue = '';
      tasks = await medicalApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openTaskStatusModal(task: any) {
    selectedTask = task;
    taskStatus = task.status ?? 'pending';
    taskProgressNotes = task.progress_notes ?? '';
    taskStatusOpen = true;
  }

  async function updateTaskStatus() {
    const s = $session; if (!s || !selectedTask) return;
    try {
      await medicalApi.updateTaskStatus(s.token, selectedTask.id, taskStatus, taskProgressNotes || undefined);
      showToast('Task updated', 'success');
      taskStatusOpen = false;
      tasks = await medicalApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function updateInventory() {
    const s = $session; if (!s) return;
    if (!invItem) { showToast('Item name required', 'error'); return; }
    try {
      await medicalApi.updateInventory(s.token, invItem, invCategory || undefined, invQty || undefined, invUnit || undefined, invExpiry ? invExpiry + 'T00:00:00Z' : undefined);
      showToast('Inventory updated', 'success');
      invOpen = false; invItem = ''; invCategory = ''; invQty = 0; invUnit = ''; invExpiry = '';
      inventory = await medicalApi.getInventory(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addSpecialization() {
    const s = $session; if (!s) return;
    if (!specStaffSelected || !specName) { showToast('Staff User and specialization required', 'error'); return; }
    try {
      await medicalApi.addSpecialization(s.token, specStaffSelected.id, specName, specDate || undefined);
      showToast('Specialization added', 'success');
      specOpen = false; specStaffSelected = null; specName = ''; specDate = '';
      specializations = await medicalApi.getSpecializations(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitBudget() {
    const s = $session; if (!s) return;
    if (!budgetTitle || !budgetDesc || !budgetAmount) { showToast('Title, description, and amount required', 'error'); return; }
    try {
      await budgetApi.submitBudgetRequest(s.token, budgetTitle, budgetDesc, budgetAmount, budgetItems || undefined);
      showToast('Budget request submitted', 'success');
      budgetOpen = false; budgetTitle = ''; budgetDesc = ''; budgetAmount = 0; budgetItems = '';
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

  const shiftCols = [
    { key: 'staff_name', label: 'Staff' },
    { key: 'shift_start', label: 'Start' },
    { key: 'shift_end', label: 'End' },
    { key: 'notes', label: 'Notes' },
  ];
  const invCols = [
    { key: 'item_name', label: 'Item' },
    { key: 'category', label: 'Category' },
    { key: 'quantity', label: 'Qty' },
    { key: 'expiry_date', label: 'Expiry' },
  ];
  const specCols = [
    { key: 'staff_name', label: 'Staff' },
    { key: 'specialization', label: 'Specialization' },
    { key: 'certified_at', label: 'Certified' },
  ];
  const taskStatusOpts = [
    { value: 'pending', label: 'Pending' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
</script>

<svelte:head><title>RUSA IMS — Medical Services</title></svelte:head>

<PageShell title="Medical Services" subtitle="Staff shifts, tasks, inventory, specializations, and budget">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='shifts'} onclick={() => activeTab='shifts'}>Shifts</button>
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>{isHead ? 'Tasks' : 'My Tasks'}</button>
    <button class="tab" class:active={activeTab==='inventory'} onclick={() => activeTab='inventory'}>Inventory</button>
    <button class="tab" class:active={activeTab==='specializations'} onclick={() => activeTab='specializations'}>Specializations</button>
    <button class="tab" class:active={activeTab==='budget'} onclick={() => activeTab='budget'}>Budget</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'shifts'}
    <div class="section-bar">
      <h2 class="section-title">Staff Shifts</h2>
      <button class="btn-primary" onclick={() => shiftOpen = true}>+ Allocate Shift</button>
    </div>
    <Table columns={shiftCols} rows={shifts} />
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">{isHead ? 'Tasks I Assigned' : 'My Assigned Tasks'}</h2>
      {#if isHead}
        <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if tasks.length === 0}
      <p class="empty">{isHead ? 'No tasks assigned yet.' : 'No tasks have been assigned to you.'}</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Progress Notes</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each tasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>{task.progress_notes ?? '—'}</td>
                <td>
                  <button class="btn-small" onclick={() => openTaskStatusModal(task)}>Update</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'inventory'}
    <div class="section-bar">
      <h2 class="section-title">Medical Inventory</h2>
      <button class="btn-primary" onclick={() => invOpen = true}>+ Update Inventory</button>
    </div>
    <Table columns={invCols} rows={inventory} />
  {:else if activeTab === 'specializations'}
    <div class="section-bar">
      <h2 class="section-title">Staff Specializations</h2>
      <button class="btn-primary" onclick={() => specOpen = true}>+ Add Specialization</button>
    </div>
    <Table columns={specCols} rows={specializations} />
  {:else if activeTab === 'budget'}
    <div class="section-bar">
      <h2 class="section-title">Budget Requests</h2>
      <button class="btn-primary" onclick={() => budgetOpen = true}>+ Submit Budget Request</button>
    </div>
    <p class="info-note">Submit budget requests for review by the finance team.</p>
  {/if}
</PageShell>

<Modal bind:open={shiftOpen} title="Allocate Shift">
  <div class="form">
    <div class="field">
      <label class="field-label">Staff User</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={shiftStaffSelected} />
    </div>
    <Field label="Shift Start" type="datetime-local" bind:value={shiftStart} required />
    <Field label="Shift End" type="datetime-local" bind:value={shiftEnd} required />
    <Field label="Notes" type="textarea" bind:value={shiftNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => shiftOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={allocateShift}>Allocate</button>
    </div>
  </div>
</Modal>

<Modal bind:open={taskOpen} title="Assign Medical Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assigneeSelected} />
    </div>
    <Field label="Title" bind:value={tTitle} required />
    <Field label="Description" type="textarea" bind:value={tDesc} />
    <Field label="Due Date" type="datetime-local" bind:value={tDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { taskOpen = false; assigneeSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={taskStatusOpen} title="Update Task Status">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedTask?.title}</strong></p>
    <Field label="Status" type="select" bind:value={taskStatus} options={taskStatusOpts} />
    <Field label="Progress Notes" type="textarea" bind:value={taskProgressNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => taskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={invOpen} title="Update Medical Inventory">
  <div class="form">
    <Field label="Item Name" bind:value={invItem} required />
    <Field label="Category" bind:value={invCategory} placeholder="e.g. medications, equipment" />
    <Field label="Quantity" type="number" bind:value={invQty} />
    <Field label="Unit" bind:value={invUnit} placeholder="e.g. units, mg, litres" />
    <Field label="Expiry Date" type="date" bind:value={invExpiry} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateInventory}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={specOpen} title="Add Specialization">
  <div class="form">
    <div class="field">
      <label class="field-label">Staff User</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={specStaffSelected} />
    </div>
    <Field label="Specialization" bind:value={specName} required placeholder="e.g. Surgery, Neurology" />
    <Field label="Certification Date" type="date" bind:value={specDate} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => specOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addSpecialization}>Add</button>
    </div>
  </div>
</Modal>

<Modal bind:open={budgetOpen} title="Submit Budget Request">
  <div class="form">
    <Field label="Title" bind:value={budgetTitle} required />
    <Field label="Description" type="textarea" bind:value={budgetDesc} required />
    <Field label="Amount (USD)" type="number" bind:value={budgetAmount} required />
    <Field label="Line Items" type="textarea" bind:value={budgetItems} hint="Itemized breakdown" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => budgetOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitBudget}>Submit</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .info-note { color: #4a5d82; font-size: 0.85rem; }
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
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-cancelled { background: rgba(255,68,102,0.12); color: #ff4466; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
</style>
