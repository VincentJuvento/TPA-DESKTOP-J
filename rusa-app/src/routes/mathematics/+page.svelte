<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { researchTaskApi, aerospaceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'assigned_tasks' | 'my_tasks' | 'pending_conclusions' | 'help_requests';
  let activeTab = $state<Tab>('assigned_tasks');

  let tasks: any[] = $state([]);
  let helpRequests: any[] = $state([]);
  let allUsers: any[] = $state([]);
  let loading = $state(false);

  // Assign task form (director / the_artificer only)
  let assignOpen = $state(false);
  let aTitle = $state('');
  let aDesc = $state('');
  let aDue = $state('');
  let assigneeSelected: any = $state(null);

  // Update task status form
  let taskStatusOpen = $state(false);
  let selectedTask: any = $state(null);
  let taskStatus = $state('pending');
  let taskProgressNotes = $state('');

  // Request conclusion form (mathematician)
  let conclusionReqOpen = $state(false);
  let conclusionReqTask: any = $state(null);
  let conclusionFinalNotes = $state('');
  let conclusionFinalFindings = $state('');
  let conclusionMethodology = $state('');
  let conclusionKeyResults = $state('');
  let conclusionRecommendations = $state('');
  let conclusionLimitations = $state('');

  // Review conclusion form (director)
  let conclusionReviewOpen = $state(false);
  let conclusionReviewTask: any = $state(null);
  let conclusionDecision = $state('approve');
  let conclusionReviewNotes = $state('');

  // Help request form
  let helpReqOpen = $state(false);
  let hrTitle = $state('');
  let hrDesc = $state('');
  let hrCategory = $state('');

  // Help request resolve form (director)
  let helpResolveOpen = $state(false);
  let helpResolveTarget: any = $state(null);
  let helpResolveStatus = $state('in_review');
  let helpResolveResponse = $state('');

  // Help request reject modal (director)
  let helpRejectOpen = $state(false);
  let helpRejectTarget: any = $state(null);
  let helpRejectReason = $state('');

  // Help request approve modal (director — convert to task)
  let helpApproveOpen = $state(false);
  let helpApproveTarget: any = $state(null);
  let helpApproveAssignee: any = $state(null);

  // Help request deliver response modal (director)
  let helpDeliverOpen = $state(false);
  let helpDeliverTarget: any = $state(null);
  let helpDeliverResponse = $state('');

  const isDirector = $derived(
    canPerform($session, 'the_artificer') ||
    canPerform($session, 'the_taskmaster') ||
    canPerform($session, 'the_observer') ||
    ($session?.tier ?? 0) >= 4
  );

  const isMathematician = $derived(canPerform($session, 'mathematician'));

  const myTasks = $derived(tasks.filter((t: any) => t.assigned_to === $session?.user_id));
  const tasksIAssigned = $derived(tasks.filter((t: any) => t.assigned_by === $session?.user_id && t.assigned_to !== $session?.user_id));
  const pendingConclusions = $derived(tasks.filter((t: any) => t.status === 'conclusion_requested'));

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [tasks, helpRequests, allUsers] = await Promise.all([
        researchTaskApi.getTasks(s.token),
        aerospaceApi.getHelpRequests(s.token),
        userApi.getAll(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!aTitle) { showToast('Title required', 'error'); return; }
    try {
      await researchTaskApi.assign(s.token, aTitle, aDesc || undefined, assigneeSelected?.id || undefined, undefined, aDue ? aDue + 'T00:00:00Z' : undefined);
      showToast('Task assigned', 'success');
      assignOpen = false; aTitle = ''; aDesc = ''; aDue = ''; assigneeSelected = null;
      tasks = await researchTaskApi.getTasks(s.token);
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
      await researchTaskApi.updateStatus(s.token, selectedTask.id, taskStatus, taskProgressNotes || undefined);
      showToast('Task updated', 'success');
      taskStatusOpen = false;
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openConclusionRequest(task: any) {
    conclusionReqTask = task;
    conclusionFinalNotes = '';
    conclusionFinalFindings = '';
    conclusionMethodology = '';
    conclusionKeyResults = '';
    conclusionRecommendations = '';
    conclusionLimitations = '';
    conclusionReqOpen = true;
  }

  async function submitConclusionRequest() {
    const s = $session; if (!s || !conclusionReqTask) return;
    if (!conclusionFinalNotes) { showToast('Final notes are required', 'error'); return; }
    try {
      await researchTaskApi.requestConclusion(
        s.token, conclusionReqTask.id, conclusionFinalNotes,
        conclusionFinalFindings || undefined, conclusionMethodology || undefined,
        conclusionKeyResults || undefined, conclusionRecommendations || undefined,
        conclusionLimitations || undefined
      );
      showToast('Conclusion request submitted', 'success');
      conclusionReqOpen = false;
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openConclusionReview(task: any) {
    conclusionReviewTask = task;
    conclusionDecision = 'approve';
    conclusionReviewNotes = '';
    conclusionReviewOpen = true;
  }

  async function submitConclusionReview() {
    const s = $session; if (!s || !conclusionReviewTask) return;
    try {
      await researchTaskApi.reviewConclusion(s.token, conclusionReviewTask.id, conclusionDecision, conclusionReviewNotes || undefined);
      showToast('Conclusion ' + (conclusionDecision === 'approve' ? 'approved' : 'rejected'), 'success');
      conclusionReviewOpen = false;
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function completeTask(taskId: string) {
    const s = $session; if (!s) return;
    try {
      await researchTaskApi.complete(s.token, taskId);
      showToast('Task marked complete', 'success');
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitHelpRequest() {
    const s = $session; if (!s) return;
    if (!hrTitle) { showToast('Title required', 'error'); return; }
    try {
      await aerospaceApi.submitHelpRequest(s.token, hrTitle, hrDesc || undefined, hrCategory || undefined);
      showToast('Help request submitted', 'success');
      helpReqOpen = false; hrTitle = ''; hrDesc = ''; hrCategory = '';
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpResolve(req: any) {
    helpResolveTarget = req;
    helpResolveStatus = 'in_review';
    helpResolveResponse = '';
    helpResolveOpen = true;
  }

  async function submitHelpResolve() {
    const s = $session; if (!s || !helpResolveTarget) return;
    try {
      await aerospaceApi.resolveHelpRequest(s.token, helpResolveTarget.id, helpResolveStatus, helpResolveResponse || undefined);
      showToast('Help request updated', 'success');
      helpResolveOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpReject(req: any) {
    helpRejectTarget = req;
    helpRejectReason = '';
    helpRejectOpen = true;
  }

  async function submitHelpReject() {
    const s = $session; if (!s || !helpRejectTarget) return;
    if (!helpRejectReason.trim()) { showToast('Rejection reason is required', 'error'); return; }
    try {
      await aerospaceApi.rejectHelpRequest(s.token, helpRejectTarget.id, helpRejectReason);
      showToast('Help request rejected', 'success');
      helpRejectOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpApprove(req: any) {
    helpApproveTarget = req;
    helpApproveAssignee = null;
    helpApproveOpen = true;
  }

  async function submitHelpApprove() {
    const s = $session; if (!s || !helpApproveTarget) return;
    if (!helpApproveAssignee) { showToast('Select a mathematician to assign the task to', 'error'); return; }
    try {
      await aerospaceApi.approveHelpRequest(s.token, helpApproveTarget.id, helpApproveAssignee.id);
      showToast('Help request approved — task created', 'success');
      helpApproveOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpDeliver(req: any) {
    helpDeliverTarget = req;
    helpDeliverResponse = '';
    helpDeliverOpen = true;
  }

  async function submitHelpDeliver() {
    const s = $session; if (!s || !helpDeliverTarget) return;
    if (!helpDeliverResponse.trim()) { showToast('Response is required', 'error'); return; }
    try {
      await aerospaceApi.proxyDeliverTaskResponse(s.token, helpDeliverTarget.id, helpDeliverResponse);
      showToast('Response delivered', 'success');
      helpDeliverOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function statusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'pending': return 'badge badge-open';
      case 'in_progress': return 'badge badge-progress';
      case 'conclusion_requested': return 'badge badge-vote';
      case 'completed': return 'badge badge-done';
      case 'cancelled': return 'badge badge-cancelled';
      case 'result_submitted': return 'badge badge-vote';
      case 'open': return 'badge badge-open';
      case 'in_review': return 'badge badge-progress';
      case 'converted': return 'badge badge-vote';
      case 'rejected': return 'badge badge-cancelled';
      default: return 'badge';
    }
  }

  const taskStatusOpts = [
    { value: 'pending', label: 'Pending' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];

  const conclusionDecisionOpts = [
    { value: 'approve', label: 'Approve' },
    { value: 'reject', label: 'Reject' },
  ];

  const helpResolveOpts = [
    { value: 'in_review', label: 'Mark In Review' },
    { value: 'closed', label: 'Close' },
  ];
</script>

<svelte:head><title>RUSA IMS — Mathematics</title></svelte:head>

<PageShell title="Mathematics" subtitle="Theoretical research tasks, conclusions, and help requests">
  <div class="tabs">
    <button class="tab" class:active={activeTab === 'assigned_tasks'} onclick={() => activeTab = 'assigned_tasks'}>
      My Assigned Tasks
    </button>
    {#if isDirector}
      <button class="tab" class:active={activeTab === 'my_tasks'} onclick={() => activeTab = 'my_tasks'}>
        Tasks I Assigned
        {#if tasksIAssigned.length > 0}
          <span class="tab-count">{tasksIAssigned.length}</span>
        {/if}
      </button>
      <button class="tab" class:active={activeTab === 'pending_conclusions'} onclick={() => activeTab = 'pending_conclusions'}>
        Pending Conclusions
        {#if pendingConclusions.length > 0}
          <span class="tab-count tab-count-alert">{pendingConclusions.length}</span>
        {/if}
      </button>
    {/if}
    <button class="tab" class:active={activeTab === 'help_requests'} onclick={() => activeTab = 'help_requests'}>Help Requests</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>

  {:else if activeTab === 'assigned_tasks'}
    <div class="section-bar">
      <h2 class="section-title">My Assigned Tasks</h2>
    </div>
    {#if myTasks.length === 0}
      <p class="empty">No tasks have been assigned to you.</p>
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
            {#each myTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td class="desc-cell">{task.progress_notes ?? '—'}</td>
                <td class="actions-cell">
                  {#if task.status !== 'completed' && task.status !== 'cancelled' && task.status !== 'conclusion_requested'}
                    <button class="btn-small" onclick={() => openTaskStatusModal(task)}>Update</button>
                  {/if}
                  {#if task.status === 'in_progress' || task.status === 'pending'}
                    <button class="btn-small btn-conclude" onclick={() => openConclusionRequest(task)}>Request Conclusion</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  {:else if activeTab === 'my_tasks'}
    <div class="section-bar">
      <h2 class="section-title">Tasks I Assigned</h2>
      {#if isDirector}
        <button class="btn-primary" onclick={() => assignOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if tasksIAssigned.length === 0}
      <p class="empty">No tasks assigned yet.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Progress Notes</th>
              <th>Final Notes</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each tasksIAssigned as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td class="desc-cell">{task.progress_notes ?? '—'}</td>
                <td class="desc-cell">{task.final_notes ?? '—'}</td>
                <td class="actions-cell">
                  {#if task.status === 'conclusion_requested'}
                    <button class="btn-small btn-approve" onclick={() => openConclusionReview(task)}>Review Conclusion</button>
                  {:else if task.status === 'result_submitted'}
                    <button class="btn-small btn-approve" onclick={() => completeTask(task.id)}>Mark Complete</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  {:else if activeTab === 'pending_conclusions'}
    <div class="section-bar">
      <h2 class="section-title">Pending Conclusion Reviews</h2>
    </div>
    {#if pendingConclusions.length === 0}
      <p class="empty">No tasks awaiting conclusion review.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Mathematician</th>
              <th>Final Notes</th>
              <th>Methodology</th>
              <th>Key Results</th>
              <th>Requested</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each pendingConclusions as task}
              <tr>
                <td>{task.title}</td>
                <td>{task.assigned_to ?? '—'}</td>
                <td class="desc-cell">{task.final_notes ?? '—'}</td>
                <td class="desc-cell">{task.methodology_summary ?? '—'}</td>
                <td class="desc-cell">{task.key_results ?? '—'}</td>
                <td>{task.conclusion_requested_at ? new Date(task.conclusion_requested_at).toLocaleDateString() : '—'}</td>
                <td>
                  <button class="btn-small btn-approve" onclick={() => openConclusionReview(task)}>Approve / Reject</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  {:else if activeTab === 'help_requests'}
    <div class="section-bar">
      <h2 class="section-title">Help Requests</h2>
      {#if isMathematician}
        <button class="btn-primary" onclick={() => helpReqOpen = true}>+ Send Help Request</button>
      {/if}
    </div>
    {#if helpRequests.length === 0}
      <p class="empty">No help requests found.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Category</th>
              <th>Routed To</th>
              <th>Status</th>
              <th>Response / Rejection</th>
              <th>Date</th>
              {#if isDirector}<th>Actions</th>{/if}
            </tr>
          </thead>
          <tbody>
            {#each helpRequests as req}
              <tr>
                <td>{req.title}</td>
                <td>{req.category ?? '—'}</td>
                <td><span class="proxy-badge">{req.assigned_proxy_director}</span></td>
                <td><span class={statusBadgeClass(req.status)}>{req.status ?? '—'}</span></td>
                <td class="desc-cell">
                  {#if req.status === 'rejected' && req.rejection_reason}
                    <span class="badge badge-cancelled" style="font-size:0.7rem">Rejected:</span> {req.rejection_reason}
                  {:else}
                    {req.response ?? '—'}
                  {/if}
                </td>
                <td>{req.created_at ? new Date(req.created_at).toLocaleDateString() : '—'}</td>
                {#if isDirector}
                  <td>
                    <div class="actions-cell">
                      {#if req.status === 'open' || req.status === 'in_review'}
                        <button class="btn-small btn-approve" onclick={() => openHelpApprove(req)}>Approve</button>
                        <button class="btn-small btn-reject" onclick={() => openHelpReject(req)}>Reject</button>
                        <button class="btn-small" onclick={() => openHelpResolve(req)}>Mark Review</button>
                      {:else if req.status === 'converted'}
                        <button class="btn-small btn-conclude" onclick={() => openHelpDeliver(req)}>Deliver Response</button>
                      {/if}
                    </div>
                  </td>
                {/if}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</PageShell>

<!-- Assign Task Modal -->
<Modal bind:open={assignOpen} title="Assign Research Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To (optional)</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assigneeSelected} />
    </div>
    <Field label="Title" bind:value={aTitle} required />
    <Field label="Description" type="textarea" bind:value={aDesc} />
    <Field label="Due Date" type="date" bind:value={aDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { assignOpen = false; assigneeSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<!-- Update Task Status Modal -->
<Modal bind:open={taskStatusOpen} title="Update Task Status">
  <div class="form">
    {#if selectedTask}
      <p class="info-text">Task: <strong>{selectedTask.title}</strong></p>
    {/if}
    <Field label="Status" type="select" bind:value={taskStatus} options={taskStatusOpts} />
    <Field label="Progress Notes" type="textarea" bind:value={taskProgressNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => taskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<!-- Request Conclusion Modal -->
<Modal bind:open={conclusionReqOpen} title="Request Task Conclusion">
  <div class="form">
    {#if conclusionReqTask}
      <p class="info-text">Task: <strong>{conclusionReqTask.title}</strong></p>
      <p class="vote-note">✅ This request routes directly to your assigned director for approval.</p>
    {/if}
    <Field label="Final Notes" type="textarea" bind:value={conclusionFinalNotes} rows={3} required />
    <Field label="Final Findings" type="textarea" bind:value={conclusionFinalFindings} rows={3} />
    <Field label="Methodology Summary" type="textarea" bind:value={conclusionMethodology} rows={2} />
    <Field label="Key Results" type="textarea" bind:value={conclusionKeyResults} rows={2} />
    <Field label="Recommendations" type="textarea" bind:value={conclusionRecommendations} rows={2} />
    <Field label="Limitations" type="textarea" bind:value={conclusionLimitations} rows={2} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => conclusionReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionRequest}>Submit Conclusion Request</button>
    </div>
  </div>
</Modal>

<!-- Review Conclusion Modal -->
<Modal bind:open={conclusionReviewOpen} title="Review Task Conclusion">
  <div class="form">
    {#if conclusionReviewTask}
      <div class="info-block">
        <p class="info-text">Task: <strong>{conclusionReviewTask.title}</strong></p>
        {#if conclusionReviewTask.final_notes}
          <p class="info-text">Final Notes: {conclusionReviewTask.final_notes}</p>
        {/if}
        {#if conclusionReviewTask.final_findings}
          <p class="info-text">Findings: {conclusionReviewTask.final_findings}</p>
        {/if}
        {#if conclusionReviewTask.methodology_summary}
          <p class="info-text">Methodology: {conclusionReviewTask.methodology_summary}</p>
        {/if}
        {#if conclusionReviewTask.key_results}
          <p class="info-text">Key Results: {conclusionReviewTask.key_results}</p>
        {/if}
        {#if conclusionReviewTask.recommendations}
          <p class="info-text">Recommendations: {conclusionReviewTask.recommendations}</p>
        {/if}
        {#if conclusionReviewTask.limitations}
          <p class="info-text">Limitations: {conclusionReviewTask.limitations}</p>
        {/if}
      </div>
    {/if}
    <Field label="Decision" type="select" bind:value={conclusionDecision} options={conclusionDecisionOpts} />
    <Field label="Review Notes (optional)" type="textarea" bind:value={conclusionReviewNotes} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => conclusionReviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<!-- Help Request Modal -->
<Modal bind:open={helpReqOpen} title="Submit Help Request">
  <div class="form">
    <p class="vote-note">ℹ️ Your help request will be routed to <strong>The Artificer</strong> for review.</p>
    <Field label="Title" bind:value={hrTitle} required />
    <Field label="Category (optional)" bind:value={hrCategory} placeholder="e.g. Data Access, Computation, Analysis" />
    <Field label="Description (optional)" type="textarea" bind:value={hrDesc} rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpRequest}>Submit</button>
    </div>
  </div>
</Modal>

<!-- Help Resolve Modal -->
<Modal bind:open={helpResolveOpen} title="Update Help Request">
  <div class="form">
    <Field label="Status" type="select" bind:value={helpResolveStatus} options={helpResolveOpts} />
    <Field label="Response (optional)" type="textarea" bind:value={helpResolveResponse} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpResolveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpResolve}>Update</button>
    </div>
  </div>
</Modal>

<!-- Help Reject Modal -->
<Modal bind:open={helpRejectOpen} title="Reject Help Request">
  <div class="form">
    <Field label="Rejection Reason" type="textarea" bind:value={helpRejectReason} rows={3} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpRejectOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpReject}>Reject</button>
    </div>
  </div>
</Modal>

<!-- Help Approve Modal -->
<Modal bind:open={helpApproveOpen} title="Approve Help Request">
  <div class="form">
    <p class="vote-note">Approving will convert this help request into a research task. Select a mathematician to assign it to.</p>
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={helpApproveAssignee} />
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { helpApproveOpen = false; helpApproveAssignee = null; }}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpApprove}>Approve & Assign</button>
    </div>
  </div>
</Modal>

<!-- Help Deliver Modal -->
<Modal bind:open={helpDeliverOpen} title="Deliver Response">
  <div class="form">
    <Field label="Response" type="textarea" bind:value={helpDeliverResponse} rows={4} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpDeliverOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpDeliver}>Deliver</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0.25rem; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab {
    background: none; border: none; border-bottom: 2px solid transparent;
    color: #4a5d82; cursor: pointer; font-family: 'Space Mono', monospace;
    font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.625rem 1rem;
    text-transform: uppercase; transition: all 0.15s; display: flex; align-items: center; gap: 0.4rem;
  }
  .tab:hover { color: #8fa3cc; }
  .tab.active { border-bottom-color: #00d4ff; color: #00d4ff; }
  .tab-count {
    background: #1e2d4a; border-radius: 10px; color: #8fa3cc;
    font-size: 0.65rem; padding: 0.1rem 0.4rem;
  }
  .tab-count-alert { background: #3d7fff33; color: #3d7fff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading, .empty { color: #4a5d82; padding: 2rem 0; }
  .table-wrap { overflow-x: auto; }
  .data-table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
  .data-table th { font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.08em; color: #4a5d82; padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #1e2d4a; }
  .data-table td { padding: 0.625rem 0.75rem; border-bottom: 1px solid #0d1526; color: #c8d8f0; vertical-align: top; }
  .data-table tbody tr:hover { background: rgba(61,127,255,0.04); }
  .desc-cell { max-width: 220px; white-space: pre-wrap; word-break: break-word; color: #8fa3cc; font-size: 0.8rem; }
  .actions-cell { display: flex; gap: 0.4rem; flex-wrap: wrap; }
  .badge { display: inline-block; border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.06em; padding: 0.15rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: #1e2d4a; color: #8fa3cc; }
  .badge-progress { background: #1a3a5c; color: #3d7fff; }
  .badge-vote { background: #2a2a5a; color: #8888ff; }
  .badge-done { background: #0d3a2a; color: #00d4a0; }
  .badge-cancelled { background: #3a1a1a; color: #ff6666; }
  .proxy-badge { background: #1e2d4a; border-radius: 3px; color: #3d7fff; font-family: 'Space Mono', monospace; font-size: 0.65rem; padding: 0.15rem 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #3d7fff; border-radius: 4px; color: #3d7fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.06em; padding: 0.25rem 0.75rem; white-space: nowrap; }
  .btn-small:hover { background: #3d7fff22; }
  .btn-approve { border-color: #00d4a0; color: #00d4a0; }
  .btn-approve:hover { background: #00d4a022; }
  .btn-reject { border-color: #ff6666; color: #ff6666; }
  .btn-reject:hover { background: #ff666622; }
  .btn-conclude { border-color: #8888ff; color: #8888ff; }
  .btn-conclude:hover { background: #8888ff22; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .info-text { color: #8fa3cc; font-size: 0.85rem; }
  .info-block { background: #0d1526; border: 1px solid #1e2d4a; border-radius: 4px; display: flex; flex-direction: column; gap: 0.5rem; padding: 0.75rem; }
  .vote-note { background: #0d1a2e; border: 1px solid #1e3a5c; border-radius: 4px; color: #8fa3cc; font-size: 0.8rem; padding: 0.625rem 0.75rem; }
</style>
