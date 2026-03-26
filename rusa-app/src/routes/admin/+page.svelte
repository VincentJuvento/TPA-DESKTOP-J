<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { userApi, adminApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'users' | 'audit' | 'directors';
  let activeTab = $state<Tab>('users');

  let users: any[] = $state([]);
  let auditLog: any[] = $state([]);
  let roles: any[] = $state([]);
  let loading = $state(false);

  // Create user form
  let createUserOpen = $state(false);
  let uUsername = $state('');
  let uEmail = $state('');
  let uPassword = $state('');
  let uFullName = $state('');
  let uRole = $state('');
  let uLocation = $state('');

  // Create director form
  let createDirOpen = $state(false);
  let dUsername = $state('');
  let dEmail = $state('');
  let dPassword = $state('');
  let dFullName = $state('');
  let dRole = $state('');

  // Terminate modal
  let terminateOpen = $state(false);
  let terminateTarget: any = $state(null);
  let terminateReason = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [users, roles, auditLog] = await Promise.all([
        userApi.getAll(s.token),
        userApi.getRoles(s.token),
        adminApi.getAuditLog(s.token, 100, 0),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function createUser() {
    const s = $session; if (!s) return;
    if (!uUsername || !uEmail || !uPassword || !uFullName || !uRole) {
      showToast('All required fields must be filled', 'error'); return;
    }
    try {
      await userApi.create(s.token, uUsername, uEmail, uPassword, uFullName, uRole, uLocation || undefined);
      showToast('User created', 'success');
      createUserOpen = false; uUsername = ''; uEmail = ''; uPassword = ''; uFullName = ''; uRole = ''; uLocation = '';
      users = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createDirector() {
    const s = $session; if (!s) return;
    if (!dUsername || !dEmail || !dPassword || !dFullName || !dRole) {
      showToast('All fields required', 'error'); return;
    }
    try {
      await adminApi.createDirector(s.token, dUsername, dEmail, dPassword, dFullName, dRole);
      showToast('Director created', 'success');
      createDirOpen = false; dUsername = ''; dEmail = ''; dPassword = ''; dFullName = ''; dRole = '';
      users = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function terminateUser() {
    const s = $session; if (!s || !terminateTarget) return;
    try {
      await adminApi.terminatePersonnel(s.token, terminateTarget.id, terminateReason || undefined);
      showToast('Personnel terminated', 'success');
      terminateOpen = false; terminateReason = '';
      users = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openTerminate(user: any) { terminateTarget = user; terminateReason = ''; terminateOpen = true; }

  let roleOptions = $derived(roles.map((r: any) => ({ value: r.name, label: r.display_name || r.name })));

  const userCols = [
    { key: 'full_name', label: 'Name' },
    { key: 'username', label: 'Username' },
    { key: 'role_display_name', label: 'Role' },
    { key: 'tier', label: 'Tier' },
    { key: 'active', label: 'Active' },
  ];
  const auditCols = [
    { key: 'action', label: 'Action' },
    { key: 'performed_by_name', label: 'By' },
    { key: 'target_name', label: 'Target' },
    { key: 'created_at', label: 'Time' },
  ];
</script>

<svelte:head><title>RUSA IMS — Administration</title></svelte:head>

<PageShell title="Administration" subtitle="User management, audit log, and director controls">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='users'} onclick={() => activeTab='users'}>Users</button>
    <button class="tab" class:active={activeTab==='directors'} onclick={() => activeTab='directors'}>Create Director</button>
    <button class="tab" class:active={activeTab==='audit'} onclick={() => activeTab='audit'}>Audit Log</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'users'}
    <div class="section-bar">
      <h2 class="section-title">All Users</h2>
      <button class="btn-primary" onclick={() => createUserOpen = true}>+ Create User</button>
    </div>
    <Table columns={userCols} rows={users} onRowClick={openTerminate} />
    <p class="hint">Click a user to terminate them</p>
  {:else if activeTab === 'directors'}
    <div class="director-card">
      <h2 class="section-title" style="margin-bottom: 1.5rem;">Create Director Account</h2>
      <Field label="Username" bind:value={dUsername} required />
      <div style="margin-top: 1rem;"><Field label="Email" type="email" bind:value={dEmail} required /></div>
      <div style="margin-top: 1rem;"><Field label="Password" type="password" bind:value={dPassword} required /></div>
      <div style="margin-top: 1rem;"><Field label="Full Name" bind:value={dFullName} required /></div>
      <div style="margin-top: 1rem;"><Field label="Role" type="select" bind:value={dRole} options={roleOptions} required /></div>
      <div class="form-actions" style="margin-top: 1.5rem;">
        <button class="btn-primary" onclick={createDirector}>Create Director</button>
      </div>
    </div>
  {:else if activeTab === 'audit'}
    <div class="section-bar">
      <h2 class="section-title">Audit Log</h2>
    </div>
    <Table columns={auditCols} rows={auditLog} />
  {/if}
</PageShell>

<Modal bind:open={createUserOpen} title="Create User">
  <div class="form">
    <Field label="Username" bind:value={uUsername} required />
    <Field label="Email" type="email" bind:value={uEmail} required />
    <Field label="Password" type="password" bind:value={uPassword} required />
    <Field label="Full Name" bind:value={uFullName} required />
    <Field label="Role" type="select" bind:value={uRole} options={roleOptions} required />
    <Field label="Location" bind:value={uLocation} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => createUserOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createUser}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={terminateOpen} title="Terminate: {terminateTarget?.full_name}">
  <div class="form">
    <div class="warning-box">⚠ This will permanently terminate {terminateTarget?.full_name}'s access. This action cannot be undone.</div>
    <Field label="Reason (optional)" type="textarea" bind:value={terminateReason} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => terminateOpen = false}>Cancel</button>
      <button class="btn-danger" onclick={terminateUser}>Terminate</button>
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
  .hint { font-size: 0.75rem; color: #4a5d82; margin-top: 0.5rem; }
  .director-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; max-width: 500px; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; }
  .warning-box { background: rgba(255,68,102,0.1); border: 1px solid rgba(255,68,102,0.3); border-radius: 4px; color: #ff4466; font-size: 0.82rem; padding: 0.75rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-danger { background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
