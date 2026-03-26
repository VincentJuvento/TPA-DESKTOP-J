<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { psychiatryApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'patients' | 'appointments' | 'tasks';
  let activeTab = $state<Tab>('patients');

  let patients: any[] = $state([]);
  let appointments: any[] = $state([]);
  let tasks: any[] = $state([]);
  let loading = $state(false);

  // Register patient
  let registerOpen = $state(false);
  let patientSelected: any = $state(null);

  // Schedule appointment
  let apptOpen = $state(false);
  let apptPatientSelected: any = $state(null);
  let apptTime = $state('');

  // Complete appointment
  let completeOpen = $state(false);
  let selectedAppt: any = $state(null);
  let findings = $state('');

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

  const isPsychiatrist = $derived(canPerform($session, 'psychiatrist'));

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [patients, appointments, tasks] = await Promise.all([
        psychiatryApi.getPatients(s.token),
        psychiatryApi.getAppointments(s.token),
        psychiatryApi.getTasks(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function registerPatient() {
    const s = $session; if (!s) return;
    if (!patientSelected) { showToast('Patient User required', 'error'); return; }
    try {
      await psychiatryApi.registerPatient(s.token, patientSelected.id);
      showToast('Patient registered', 'success');
      registerOpen = false; patientSelected = null;
      patients = await psychiatryApi.getPatients(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function scheduleAppointment() {
    const s = $session; if (!s) return;
    if (!apptPatientSelected || !apptTime) { showToast('Patient and time required', 'error'); return; }
    try {
      await psychiatryApi.scheduleAppointment(s.token, apptPatientSelected.id, apptTime);
      showToast('Appointment scheduled', 'success');
      apptOpen = false; apptPatientSelected = null; apptTime = '';
      appointments = await psychiatryApi.getAppointments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function completeAppointment() {
    const s = $session; if (!s || !selectedAppt) return;
    try {
      await psychiatryApi.completeAppointment(s.token, selectedAppt.id, findings || undefined);
      showToast('Appointment completed', 'success');
      completeOpen = false; findings = '';
      appointments = await psychiatryApi.getAppointments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !tTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await psychiatryApi.assignTask(s.token, assigneeSelected.id, tTitle, tDesc || undefined, tDue || undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; assigneeSelected = null; tTitle = ''; tDesc = ''; tDue = '';
      tasks = await psychiatryApi.getTasks(s.token);
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
      await psychiatryApi.updateTaskStatus(s.token, selectedTask.id, taskStatus, taskProgressNotes || undefined);
      showToast('Task updated', 'success');
      taskStatusOpen = false;
      tasks = await psychiatryApi.getTasks(s.token);
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

  const patientCols = [
    { key: 'patient_name', label: 'Patient' },
    { key: 'registered_at', label: 'Registered' },
    { key: 'status', label: 'Status' },
  ];
  const apptCols = [
    { key: 'patient_name', label: 'Patient' },
    { key: 'scheduled_at', label: 'Scheduled' },
    { key: 'status', label: 'Status' },
  ];
  const taskStatusOpts = [
    { value: 'pending', label: 'Pending' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
</script>

<svelte:head><title>RUSA IMS — Psychiatry</title></svelte:head>

<PageShell title="Psychiatry" subtitle="Patient management, appointments, and task assignments">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='patients'} onclick={() => activeTab='patients'}>Patients</button>
    <button class="tab" class:active={activeTab==='appointments'} onclick={() => activeTab='appointments'}>Appointments</button>
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>{isPsychiatrist ? 'Tasks' : 'My Tasks'}</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'patients'}
    <div class="section-bar">
      <h2 class="section-title">Patients</h2>
      <button class="btn-primary" onclick={() => registerOpen = true}>+ Register Patient</button>
    </div>
    <Table columns={patientCols} rows={patients} />
  {:else if activeTab === 'appointments'}
    <div class="section-bar">
      <h2 class="section-title">Appointments</h2>
      <button class="btn-primary" onclick={() => apptOpen = true}>+ Schedule Appointment</button>
    </div>
    <Table columns={apptCols} rows={appointments} onRowClick={(r) => { selectedAppt = r; findings = ''; completeOpen = true; }} />
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">{isPsychiatrist ? 'Tasks I Assigned' : 'My Assigned Tasks'}</h2>
      {#if isPsychiatrist}
        <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if tasks.length === 0}
      <p class="empty">{isPsychiatrist ? 'No tasks assigned yet.' : 'No tasks have been assigned to you.'}</p>
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
  {/if}
</PageShell>

<Modal bind:open={registerOpen} title="Register Patient">
  <div class="form">
    <div class="field">
      <label class="field-label">Patient User</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={patientSelected} />
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { registerOpen = false; patientSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={registerPatient}>Register</button>
    </div>
  </div>
</Modal>

<Modal bind:open={apptOpen} title="Schedule Appointment">
  <div class="form">
    <div class="field">
      <label class="field-label">Patient User</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={apptPatientSelected} />
    </div>
    <Field label="Scheduled Time" type="datetime-local" bind:value={apptTime} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { apptOpen = false; apptPatientSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={scheduleAppointment}>Schedule</button>
    </div>
  </div>
</Modal>

<Modal bind:open={completeOpen} title="Complete Appointment">
  <div class="form">
    <p class="info-text">Patient: <strong>{selectedAppt?.patient_name}</strong></p>
    <Field label="Findings / Notes" type="textarea" bind:value={findings} rows={5} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => completeOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={completeAppointment}>Mark Complete</button>
    </div>
  </div>
</Modal>

<Modal bind:open={taskOpen} title="Assign Psychiatry Task">
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

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
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
