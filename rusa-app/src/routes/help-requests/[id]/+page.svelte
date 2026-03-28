<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import PageShell from '$lib/components/PageShell.svelte';
  import WorkspaceDetail from '$lib/components/WorkspaceDetail.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { aerospaceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  const reqId = $derived($page.params.id);
  let request: any = $state(null);
  let allUsers: any[] = $state([]);
  let loading = $state(true);
  let activeTab = $state<'overview' | 'activity'>('overview');

  const isObserver = $derived(canPerform($session, 'the_observer'));
  const roleHint = $derived($session?.role_name === 'chemist' || $session?.role_name === 'physicist' ? 'chemistry' : 'research');
  const backPath = $derived(roleHint === 'chemistry' ? '/chemistry' : '/research');

  let helpResolveOpen = $state(false);
  let helpResolveStatus = $state('in_review');
  let helpResolveResponse = $state('');
  let helpRejectOpen = $state(false);
  let helpRejectReason = $state('');
  let helpApproveOpen = $state(false);
  let helpApproveAssignee: any = $state(null);
  let helpDeliverOpen = $state(false);
  let helpDeliverResponse = $state('');

  onMount(async () => {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      const [requests, users] = await Promise.all([
        aerospaceApi.getHelpRequests(s.token),
        userApi.getAll(s.token),
      ]);
      request = requests.find((r: any) => String(r.id) === String(reqId)) ?? null;
      allUsers = users;
      if (!request) {
        showToast('Help request not found', 'error');
        goto('/research');
      }
    } catch (e: any) {
      showToast('Failed to load: ' + e, 'error');
    }
    loading = false;
  });

  async function refresh() {
    const s = $session; if (!s) return;
    const requests = await aerospaceApi.getHelpRequests(s.token);
    request = requests.find((r: any) => String(r.id) === String(reqId)) ?? null;
  }

  const statusClass = (status: string | undefined) =>
    status === 'resolved' || status === 'closed' ? 'badge badge-done'
    : status === 'rejected' ? 'badge badge-rejected'
    : status === 'converted' ? 'badge badge-progress'
    : 'badge badge-open';

  const workspaceTags = $derived([
    { label: request?.status ?? '—', className: statusClass(request?.status) },
    ...(request?.category ? [{ label: `📂 ${request.category}` }] : []),
    ...(request?.assigned_proxy_director ? [{ label: `👤 ${request.assigned_proxy_director}` }] : []),
    ...(request?.created_at ? [{ label: `📅 ${new Date(request.created_at).toLocaleDateString()}` }] : []),
  ]);

  const workspaceActions = $derived([
    ...(isObserver && (request?.status === 'open' || request?.status === 'in_review')
      ? [
          { label: 'Mark Review', className: 'btn-secondary', onClick: () => { helpResolveStatus = 'in_review'; helpResolveResponse = ''; helpResolveOpen = true; } },
          { label: 'Approve', className: 'btn-primary', onClick: () => { helpApproveAssignee = null; helpApproveOpen = true; } },
          { label: 'Reject', className: 'btn-danger', onClick: () => { helpRejectReason = ''; helpRejectOpen = true; } },
        ]
      : []),
    ...(isObserver && request?.status === 'converted'
      ? [{ label: 'Deliver Response', className: 'btn-observer', onClick: () => { helpDeliverResponse = ''; helpDeliverOpen = true; } }]
      : []),
  ]);
</script>

<svelte:head><title>RUSA IMS — {request?.title ?? 'Help Request'}</title></svelte:head>

<PageShell title={request?.title ?? 'Help Request'} subtitle="Help Request Workspace">
  {#if loading}
    <p class="loading">Loading help request…</p>
  {:else if !request}
    <p class="empty">Help request not found.</p>
  {:else}
    <WorkspaceDetail
      backLabel="Back"
      onBack={() => goto(backPath)}
      tags={workspaceTags}
      actions={workspaceActions}
      tabs={[{ key: 'overview', label: 'Overview' }, { key: 'activity', label: 'Activity' }]}
      activeTab={activeTab}
      onSelectTab={(t) => activeTab = t as 'overview' | 'activity'}
    >
      {#snippet body()}
        {#if activeTab === 'overview'}
          <div class="overview-grid">
            <div class="detail-card full-width">
              <h3 class="detail-label">Request Description</h3>
              <div class="readonly-block">{request.description ?? '—'}</div>
            </div>
            <div class="detail-row">
              <div class="detail-card">
                <h3 class="detail-label">Category</h3>
                <p class="detail-value">{request.category ?? '—'}</p>
              </div>
              <div class="detail-card">
                <h3 class="detail-label">Status</h3>
                <p class="detail-value">{request.status ?? '—'}</p>
              </div>
              <div class="detail-card">
                <h3 class="detail-label">Proxy Director</h3>
                <p class="detail-value">{request.assigned_proxy_director ?? '—'}</p>
              </div>
              <div class="detail-card">
                <h3 class="detail-label">Created</h3>
                <p class="detail-value">{request.created_at ? new Date(request.created_at).toLocaleString() : '—'}</p>
              </div>
            </div>
          </div>
        {:else}
          <div class="overview-grid">
            {#if request.response}
              <div class="detail-card full-width">
                <h3 class="detail-label">Latest Response</h3>
                <div class="readonly-block">{request.response}</div>
              </div>
            {/if}
            {#if request.rejection_reason}
              <div class="detail-card full-width">
                <h3 class="detail-label">Rejection Reason</h3>
                <div class="readonly-block">{request.rejection_reason}</div>
              </div>
            {/if}
            {#if request.created_task_id}
              <div class="detail-card full-width">
                <h3 class="detail-label">Linked Task</h3>
                <p class="detail-value"><code>{request.created_task_id}</code></p>
              </div>
            {/if}
            {#if !request.response && !request.rejection_reason && !request.created_task_id}
              <p class="empty">No activity yet for this help request.</p>
            {/if}
          </div>
        {/if}
      {/snippet}
    </WorkspaceDetail>
  {/if}
</PageShell>

<Modal bind:open={helpResolveOpen} title="Mark Help Request">
  <div class="form">
    <Field label="Status" type="select" bind:value={helpResolveStatus} options={[{ value: 'in_review', label: 'Mark In Review' }, { value: 'closed', label: 'Close' }]} />
    <Field label="Notes (optional)" type="textarea" bind:value={helpResolveResponse} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpResolveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={async () => {
        const s = $session; if (!s || !request) return;
        try {
          await aerospaceApi.resolveHelpRequest(s.token, request.id, helpResolveStatus, helpResolveResponse || undefined);
          showToast('Help request updated', 'success');
          helpResolveOpen = false;
          await refresh();
        } catch (e: any) { showToast('Failed: ' + e, 'error'); }
      }}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpRejectOpen} title="Reject Help Request">
  <div class="form">
    <Field label="Rejection Reason" type="textarea" bind:value={helpRejectReason} rows={4} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpRejectOpen = false}>Cancel</button>
      <button class="btn-danger" onclick={async () => {
        const s = $session; if (!s || !request) return;
        if (!helpRejectReason.trim()) { showToast('Rejection reason required', 'error'); return; }
        try {
          await aerospaceApi.rejectHelpRequest(s.token, request.id, helpRejectReason);
          showToast('Help request rejected', 'success');
          helpRejectOpen = false;
          await refresh();
        } catch (e: any) { showToast('Failed: ' + e, 'error'); }
      }} disabled={!helpRejectReason.trim()}>Reject Request</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpApproveOpen} title="Approve Help Request — Assign Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign Task To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={helpApproveAssignee} />
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpApproveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={async () => {
        const s = $session; if (!s || !request) return;
        if (!helpApproveAssignee) { showToast('Assignee required', 'error'); return; }
        try {
          await aerospaceApi.approveHelpRequest(s.token, request.id, helpApproveAssignee.id);
          showToast('Help request approved — task created', 'success');
          helpApproveOpen = false;
          await refresh();
        } catch (e: any) { showToast('Failed: ' + e, 'error'); }
      }} disabled={!helpApproveAssignee}>Approve & Create Task</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpDeliverOpen} title="Deliver Task Response">
  <div class="form">
    <Field label="Response / Delivery Notes" type="textarea" bind:value={helpDeliverResponse} rows={4} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpDeliverOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={async () => {
        const s = $session; if (!s || !request) return;
        if (!helpDeliverResponse.trim()) { showToast('Response required', 'error'); return; }
        try {
          await aerospaceApi.proxyDeliverTaskResponse(s.token, request.id, helpDeliverResponse);
          showToast('Response delivered', 'success');
          helpDeliverOpen = false;
          await refresh();
        } catch (e: any) { showToast('Failed: ' + e, 'error'); }
      }} disabled={!helpDeliverResponse.trim()}>Deliver Response</button>
    </div>
  </div>
</Modal>

<style>
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .overview-grid { display: flex; flex-direction: column; gap: 1rem; }
  .detail-row { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 0.75rem; }
  .detail-card { background: rgba(13,21,40,0.6); border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.75rem 1rem; }
  .full-width { grid-column: 1 / -1; }
  .detail-label { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; margin: 0 0 0.35rem; text-transform: uppercase; }
  .detail-value { color: #c8d8f0; font-size: 0.9rem; margin: 0; }
  .readonly-block { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #c8d8f0; font-size: 0.85rem; line-height: 1.6; min-height: 60px; padding: 0.625rem 0.875rem; white-space: pre-wrap; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-observer { background: none; border: 1px solid #f59e0b; border-radius: 4px; color: #f59e0b; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-danger { background: linear-gradient(135deg, #dc2626, #ef4444); border: none; border-radius: 4px; color: #fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
