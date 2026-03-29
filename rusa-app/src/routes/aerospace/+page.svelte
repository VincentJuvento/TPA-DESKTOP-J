<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { aerospaceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'work_orders' | 'assigned_tasks' | 'technical_reports' | 'blueprint_proposals' | 'ships' | 'help_requests';
  let activeTab = $state<Tab>('work_orders');

  let workOrders: any[] = $state([]);
  let assignedTasks: any[] = $state([]);
  let technicalReports: any[] = $state([]);
  let blueprintProposals: any[] = $state([]);
  let ships: any[] = $state([]);
  let helpRequests: any[] = $state([]);
  let loading = $state(false);

  // New work order form
  let woOpen = $state(false);
  let woTitle = $state('');
  let woDesc = $state('');
  let woPriority = $state('medium');
  let woSystem = $state('');

  // Update work order status form
  let statusOpen = $state(false);
  let selectedOrderId = $state('');
  let newStatus = $state('open');
  let statusNotes = $state('');

  // Assign task form (manager/director)
  let assignOpen = $state(false);
  let aTitle = $state('');
  let aDesc = $state('');
  let aDue = $state('');

  let allUsers: any[] = $state([]);
  let assigneeSelected: any = $state(null);

  // Update assigned task status
  let taskStatusOpen = $state(false);
  let selectedTask: any = $state(null);
  let taskStatus = $state('pending');
  let taskProgressNotes = $state('');

  // Task conclusion request form (aerospace engineer)
  let conclusionReqOpen = $state(false);
  let conclusionReqTask: any = $state(null);
  let conclusionFinalNotes = $state('');
  let conclusionFinalFindings = $state('');
  let conclusionMethodology = $state('');
  let conclusionKeyResults = $state('');
  let conclusionRecommendations = $state('');
  let conclusionLimitations = $state('');

  // Task conclusion approval form (director / taskmaster)
  let conclusionApproveOpen = $state(false);
  let conclusionApproveTask: any = $state(null);
  let conclusionDecision = $state('approve');
  let conclusionReviewNotes = $state('');

  // Help request form
  let helpReqOpen = $state(false);
  let hrTitle = $state('');
  let hrDesc = $state('');
  let hrCategory = $state('');

  // Help request resolve form (director — generic: in_review / closed)
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

  // Help request deliver response modal (director — proxy deliver)
  let helpDeliverOpen = $state(false);
  let helpDeliverTarget: any = $state(null);
  let helpDeliverResponse = $state('');

  // Ship detail modal
  let shipDetailOpen = $state(false);
  let shipDetailData: any = $state(null);
  let shipFilterStatus = $state('');

  // Technical report form
  let reportOpen = $state(false);
  let rTitle = $state('');
  let rContent = $state('');
  let rFindings = $state('');
  let rRecommendations = $state('');

  // Blueprint proposal form
  let blueprintOpen = $state(false);
  let bpShipName = $state('');
  let bpDescription = $state('');
  let bpDesignSpecs = $state('');

  // Blueprint review form (directors)
  let bpReviewOpen = $state(false);
  let bpReviewTarget: any = $state(null);
  let bpReviewStatus = $state('approved');
  let bpReviewNotes = $state('');

  const canAssignTasks = $derived(($session?.tier ?? 0) >= 2);
  const isDirector = $derived(($session?.tier ?? 0) >= 3);
  const isEngineer = $derived(canPerform($session, 'aerospace_engineer'));

  const myAerospaceTasks = $derived(assignedTasks.filter((t: any) => t.assigned_to === $session?.user_id));
  const assignedByMeTasks = $derived(assignedTasks.filter((t: any) => t.assigned_by === $session?.user_id && t.assigned_to !== $session?.user_id));

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [workOrders, assignedTasks, technicalReports, blueprintProposals, ships, helpRequests] = await Promise.all([
        aerospaceApi.getWorkOrders(s.token),
        aerospaceApi.getAssignedTasks(s.token),
        aerospaceApi.getTechnicalReports(s.token),
        aerospaceApi.getBlueprintProposals(s.token),
        aerospaceApi.getAllShips(s.token),
        aerospaceApi.getHelpRequests(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function createWorkOrder() {
    const s = $session; if (!s) return;
    if (!woTitle) { showToast('Title required', 'error'); return; }
    try {
      await aerospaceApi.createWorkOrder(s.token, woTitle, woDesc || undefined, woPriority || undefined, woSystem || undefined);
      showToast('Work order created', 'success');
      woOpen = false; woTitle = ''; woDesc = ''; woPriority = 'medium'; woSystem = '';
      workOrders = await aerospaceApi.getWorkOrders(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openStatusModal(order: any) {
    selectedOrderId = order.id;
    newStatus = order.status ?? 'open';
    statusNotes = order.notes ?? '';
    statusOpen = true;
  }

  async function updateStatus() {
    const s = $session; if (!s) return;
    try {
      await aerospaceApi.updateWorkOrderStatus(s.token, selectedOrderId, newStatus, statusNotes || undefined);
      showToast('Status updated', 'success');
      statusOpen = false;
      workOrders = await aerospaceApi.getWorkOrders(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !aTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await aerospaceApi.assignTask(s.token, assigneeSelected.id, aTitle, aDesc || undefined, aDue || undefined);
      showToast('Task assigned', 'success');
      assignOpen = false; assigneeSelected = null; aTitle = ''; aDesc = ''; aDue = '';
      assignedTasks = await aerospaceApi.getAssignedTasks(s.token);
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
      await aerospaceApi.updateTaskStatus(s.token, selectedTask.id, taskStatus, taskProgressNotes || undefined);
      showToast('Task updated', 'success');
      taskStatusOpen = false;
      assignedTasks = await aerospaceApi.getAssignedTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitReport() {
    const s = $session; if (!s) return;
    if (!rTitle || !rContent) { showToast('Title and content required', 'error'); return; }
    try {
      await aerospaceApi.submitTechnicalReport(s.token, rTitle, rContent, rFindings || undefined, rRecommendations || undefined);
      showToast('Report submitted', 'success');
      reportOpen = false; rTitle = ''; rContent = ''; rFindings = ''; rRecommendations = '';
      technicalReports = await aerospaceApi.getTechnicalReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitBlueprintProposal() {
    const s = $session; if (!s) return;
    if (!bpShipName || !bpDescription) { showToast('Ship name and description required', 'error'); return; }
    try {
      await aerospaceApi.submitBlueprintProposal(s.token, bpShipName, bpDescription, bpDesignSpecs || undefined);
      showToast('Blueprint proposal submitted — directors\' vote initiated', 'success');
      blueprintOpen = false; bpShipName = ''; bpDescription = ''; bpDesignSpecs = '';
      blueprintProposals = await aerospaceApi.getBlueprintProposals(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openBlueprintReview(proposal: any) {
    bpReviewTarget = proposal;
    bpReviewStatus = 'approved';
    bpReviewNotes = '';
    bpReviewOpen = true;
  }

  async function submitBlueprintReview() {
    const s = $session; if (!s || !bpReviewTarget) return;
    try {
      await aerospaceApi.reviewBlueprintProposal(s.token, bpReviewTarget.id, bpReviewStatus, bpReviewNotes || undefined);
      showToast('Review submitted', 'success');
      bpReviewOpen = false;
      blueprintProposals = await aerospaceApi.getBlueprintProposals(s.token);
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
      await aerospaceApi.requestTaskConclusion(s.token, conclusionReqTask.id, conclusionFinalNotes, conclusionFinalFindings || undefined, conclusionMethodology || undefined, conclusionKeyResults || undefined, conclusionRecommendations || undefined, conclusionLimitations || undefined);
      showToast('Conclusion request submitted', 'success');
      conclusionReqOpen = false;
      assignedTasks = await aerospaceApi.getAssignedTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openConclusionApproval(task: any) {
    conclusionApproveTask = task;
    conclusionDecision = 'approve';
    conclusionReviewNotes = '';
    conclusionApproveOpen = true;
  }

  async function submitConclusionApproval() {
    const s = $session; if (!s || !conclusionApproveTask) return;
    try {
      await aerospaceApi.approveTaskConclusion(s.token, conclusionApproveTask.id, conclusionDecision, conclusionReviewNotes || undefined);
      showToast('Conclusion ' + (conclusionDecision === 'approve' ? 'approved' : 'rejected'), 'success');
      conclusionApproveOpen = false;
      assignedTasks = await aerospaceApi.getAssignedTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openShipDetail(ship: any) {
    const s = $session; if (!s) return;
    try {
      shipDetailData = await aerospaceApi.getShipDetails(s.token, ship.id);
      shipDetailOpen = true;
    } catch (e: any) { showToast('Failed to load ship details: ' + e, 'error'); }
  }

  const filteredShips = $derived(shipFilterStatus
    ? ships.filter(s => s.status === shipFilterStatus || s.build_status === shipFilterStatus)
    : ships);

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
    if (!helpApproveAssignee) { showToast('Select a subordinate to assign the task to', 'error'); return; }
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
      showToast('Response delivered to original requester', 'success');
      helpDeliverOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function statusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'open': return 'badge badge-open';
      case 'in_progress': return 'badge badge-progress';
      case 'pending': return 'badge badge-open';
      case 'under_vote': return 'badge badge-vote';
      case 'approved': return 'badge badge-done';
      case 'completed': return 'badge badge-done';
      case 'conclusion_requested': return 'badge badge-vote';
      case 'building': return 'badge badge-progress';
      case 'rejected': return 'badge badge-cancelled';
      case 'cancelled': return 'badge badge-cancelled';
      default: return 'badge';
    }
  }

  const priorityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
  const statusOpts = [
    { value: 'open', label: 'Open' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
  const taskStatusOpts = [
    { value: 'pending', label: 'Pending' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
  const bpReviewOpts = [
    { value: 'approved', label: 'Approve' },
    { value: 'rejected', label: 'Reject' },
  ];
  const conclusionDecisionOpts = [
    { value: 'approve', label: 'Approve' },
    { value: 'reject', label: 'Reject' },
  ];
  const helpResolveOpts = [
    { value: 'in_review', label: 'In Review' },
    { value: 'closed', label: 'Closed' },
  ];
  const shipStatusFilterOpts = [
    { value: '', label: 'All Statuses' },
    { value: 'design', label: 'Design' },
    { value: 'building', label: 'Building' },
    { value: 'available', label: 'Available' },
    { value: 'completed', label: 'Completed' },
    { value: 'in_mission', label: 'In Mission' },
    { value: 'maintenance', label: 'Maintenance' },
    { value: 'decommissioned', label: 'Decommissioned' },
  ];
  const taskCols = [
    { key: 'title', label: 'Title' },
    { key: 'status', label: 'Status' },
    { key: 'due_date', label: 'Due Date' },
    { key: 'progress_notes', label: 'Progress Notes' },
  ];
</script>

<svelte:head><title>RUSA IMS — Aerospace Engineering</title></svelte:head>

<PageShell title="Aerospace Engineering" subtitle="Work orders, assigned tasks, technical reports, blueprint proposals, ships, and help requests">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='work_orders'} onclick={() => activeTab='work_orders'}>Work Orders</button>
    <button class="tab" class:active={activeTab==='assigned_tasks'} onclick={() => activeTab='assigned_tasks'}>
      {canAssignTasks ? 'Assigned Tasks' : 'My Assigned Tasks'}
    </button>
    <button class="tab" class:active={activeTab==='technical_reports'} onclick={() => activeTab='technical_reports'}>Technical Reports</button>
    <button class="tab" class:active={activeTab==='blueprint_proposals'} onclick={() => activeTab='blueprint_proposals'}>Blueprint Proposals</button>
    <button class="tab" class:active={activeTab==='ships'} onclick={() => activeTab='ships'}>Ships</button>
    <button class="tab" class:active={activeTab==='help_requests'} onclick={() => activeTab='help_requests'}>Help Requests</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'work_orders'}
    <div class="section-bar">
      <h2 class="section-title">Work Orders</h2>
      {#if isEngineer}
        <button class="btn-primary" onclick={() => woOpen = true}>+ New Work Order</button>
      {/if}
    </div>
    <div class="table-wrap">
      <table class="data-table">
        <thead>
          <tr>
            <th>Title</th>
            <th>Priority</th>
            <th>System</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each workOrders as order}
            <tr>
              <td>{order.title}</td>
              <td>{order.priority ?? '—'}</td>
              <td>{order.system_affected ?? '—'}</td>
              <td><span class={statusBadgeClass(order.status)}>{order.status ?? '—'}</span></td>
              <td>
                {#if isEngineer}
                  <button class="btn-small" onclick={() => openStatusModal(order)}>Update Status</button>
                {/if}
              </td>
            </tr>
          {/each}
          {#if workOrders.length === 0}
            <tr><td colspan="5" class="empty">No work orders found.</td></tr>
          {/if}
        </tbody>
      </table>
    </div>

  {:else if activeTab === 'assigned_tasks'}
    <div class="section-bar">
      <h2 class="section-title">My Assigned Tasks</h2>
      {#if canAssignTasks}
        <button class="btn-primary" onclick={() => assignOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if myAerospaceTasks.length === 0}
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
            {#each myAerospaceTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>{task.progress_notes ?? '—'}</td>
                <td class="actions-cell">
                  <button class="btn-small" onclick={() => openTaskStatusModal(task)}>Update</button>
                  {#if isEngineer && (task.status === 'in_progress' || task.status === 'pending')}
                    <button class="btn-small btn-conclude" onclick={() => openConclusionRequest(task)}>Request Conclusion</button>
                  {/if}
                  {#if isDirector && task.status === 'conclusion_requested'}
                    <button class="btn-small btn-approve" onclick={() => openConclusionApproval(task)}>Approve/Reject</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    {#if canAssignTasks}
      <div class="section-bar" style="margin-top: 2rem;">
        <h2 class="section-title">Tasks I Assigned</h2>
      </div>
      {#if assignedByMeTasks.length === 0}
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
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each assignedByMeTasks as task}
                <tr>
                  <td>{task.title}</td>
                  <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                  <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                  <td>{task.progress_notes ?? '—'}</td>
                  <td class="actions-cell">
                    {#if isDirector && task.status === 'conclusion_requested'}
                      <button class="btn-small btn-approve" onclick={() => openConclusionApproval(task)}>Approve/Reject</button>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}

  {:else if activeTab === 'technical_reports'}
    <div class="section-bar">
      <h2 class="section-title">Technical Reports</h2>
      {#if isEngineer}
        <button class="btn-primary" onclick={() => reportOpen = true}>+ Submit Report</button>
      {/if}
    </div>
    <div class="reports-list">
      {#each technicalReports as report}
        <div class="report-card">
          <div class="report-header">
            <span class="report-title">{report.title}</span>
            <span class="report-date">{report.created_at ? new Date(report.created_at).toLocaleDateString() : '—'}</span>
          </div>
          {#if report.content}
            <p class="report-body">{report.content}</p>
          {/if}
          {#if report.findings}
            <div class="report-section"><span class="report-label">Findings:</span> {report.findings}</div>
          {/if}
          {#if report.recommendations}
            <div class="report-section"><span class="report-label">Recommendations:</span> {report.recommendations}</div>
          {/if}
        </div>
      {/each}
      {#if technicalReports.length === 0}
        <p class="empty">No technical reports found.</p>
      {/if}
    </div>

  {:else if activeTab === 'blueprint_proposals'}
    <div class="section-bar">
      <h2 class="section-title">Blueprint Proposals</h2>
      <div class="bypass-info-row">
        <span class="vote-type-badge vote-type-blueprint">🗳 Directors' Vote Required</span>
      </div>
      {#if isEngineer}
        <button class="btn-primary" onclick={() => blueprintOpen = true}>+ Submit Blueprint</button>
      {/if}
    </div>
    <p class="vote-system-note">
      Blueprint proposals automatically trigger a directors' vote (8-vote quorum). A ship cannot enter
      the <strong>building</strong> phase until its blueprint proposal is <strong>approved</strong>.
    </p>
    {#if blueprintProposals.length === 0}
      <p class="empty">No blueprint proposals found.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Ship Name</th>
              <th>Description</th>
              <th>Status</th>
              <th>Vote Linked</th>
              <th>Submitted</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each blueprintProposals as bp}
              <tr>
                <td>{bp.ship_name}</td>
                <td class="desc-cell">{bp.blueprint_description}</td>
                <td><span class={statusBadgeClass(bp.status)}>{bp.status ?? '—'}</span></td>
                <td>
                  {#if bp.vote_id}
                    <span class="vote-badge vote-badge-linked">🗳 Linked</span>
                  {:else}
                    <span class="vote-badge">—</span>
                  {/if}
                </td>
                <td>{bp.created_at ? new Date(bp.created_at).toLocaleDateString() : '—'}</td>
                <td>
                  {#if isDirector && (bp.status === 'under_vote' || bp.status === 'pending')}
                    <button class="btn-small" onclick={() => openBlueprintReview(bp)}>Review</button>
                  {/if}
                  {#if bp.review_notes}
                    <span class="review-notes" title={bp.review_notes}>📝</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  {:else if activeTab === 'ships'}
    <div class="section-bar">
      <h2 class="section-title">Ships Archive</h2>
      <div class="filter-row">
        <select class="filter-select" bind:value={shipFilterStatus}>
          {#each shipStatusFilterOpts as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>
    </div>
    {#if filteredShips.length === 0}
      <p class="empty">No ships found.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Ship Name</th>
              <th>Type</th>
              <th>Status</th>
              <th>Build Status</th>
              <th>Capacity</th>
              <th>Launch Date</th>
              <th>Last Updated</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredShips as ship}
              <tr>
                <td>{ship.ship_name ?? ship.name}</td>
                <td>{ship.ship_type ?? ship.model ?? '—'}</td>
                <td><span class={statusBadgeClass(ship.status)}>{ship.status ?? '—'}</span></td>
                <td>
                  {#if ship.build_status}
                    <span class={statusBadgeClass(ship.build_status)}>{ship.build_status}</span>
                  {:else}
                    —
                  {/if}
                </td>
                <td>{ship.capacity ?? '—'}</td>
                <td>{ship.launch_date ? new Date(ship.launch_date).toLocaleDateString() : '—'}</td>
                <td>{ship.last_updated ? new Date(ship.last_updated).toLocaleDateString() : '—'}</td>
                <td>
                  <button class="btn-small" onclick={() => openShipDetail(ship)}>Details</button>
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
      {#if isEngineer}
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
              <th>Response / Rejection Reason</th>
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

<Modal bind:open={woOpen} title="New Work Order">
  <div class="form">
    <Field label="Title" bind:value={woTitle} required />
    <Field label="Description" type="textarea" bind:value={woDesc} />
    <Field label="Priority" type="select" bind:value={woPriority} options={priorityOpts} />
    <Field label="System Affected" bind:value={woSystem} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => woOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createWorkOrder}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={statusOpen} title="Update Work Order Status">
  <div class="form">
    <Field label="Status" type="select" bind:value={newStatus} options={statusOpts} />
    <Field label="Notes" type="textarea" bind:value={statusNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => statusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={assignOpen} title="Assign Aerospace Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assigneeSelected} />
    </div>
    <Field label="Title" bind:value={aTitle} required />
    <Field label="Description" type="textarea" bind:value={aDesc} />
    <Field label="Due Date" type="datetime-local" bind:value={aDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { assignOpen = false; assigneeSelected = null; }}>Cancel</button>
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

<Modal bind:open={reportOpen} title="Submit Technical Report">
  <div class="form">
    <Field label="Title" bind:value={rTitle} required />
    <Field label="Content" type="textarea" bind:value={rContent} required />
    <Field label="Findings" type="textarea" bind:value={rFindings} />
    <Field label="Recommendations" type="textarea" bind:value={rRecommendations} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reportOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReport}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={blueprintOpen} title="Submit Blueprint Proposal">
  <div class="form">
    <p class="vote-system-note">
      ⚠️ Submitting a blueprint proposal will automatically initiate a directors' vote (8-vote quorum required).
      The ship cannot enter the <strong>building</strong> phase until this proposal is approved.
    </p>
    <Field label="Ship Name" bind:value={bpShipName} required />
    <Field label="Blueprint Description" type="textarea" bind:value={bpDescription} rows={4} required />
    <Field label="Design Specifications (optional)" type="textarea" bind:value={bpDesignSpecs} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => blueprintOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitBlueprintProposal}>Submit for Director Vote</button>
    </div>
  </div>
</Modal>

<Modal bind:open={bpReviewOpen} title="Review Blueprint Proposal">
  <div class="form">
    {#if bpReviewTarget}
      <div class="info-block">
        <p class="info-text">Ship: <strong>{bpReviewTarget.ship_name}</strong></p>
        <p class="info-text">Description: {bpReviewTarget.blueprint_description}</p>
        {#if bpReviewTarget.design_specs}
          <p class="info-text">Specs: {bpReviewTarget.design_specs}</p>
        {/if}
        <p class="vote-system-note">⚖️ You may only approve this proposal after the linked directors' vote has passed.</p>
      </div>
    {/if}
    <Field label="Decision" type="select" bind:value={bpReviewStatus} options={bpReviewOpts} />
    <Field label="Review Notes (optional)" type="textarea" bind:value={bpReviewNotes} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => bpReviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitBlueprintReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<Modal bind:open={conclusionReqOpen} title="Request Task Conclusion">
  <div class="form">
    {#if conclusionReqTask}
      <p class="info-text">Task: <strong>{conclusionReqTask.title}</strong></p>
      <p class="vote-system-note">✅ This request bypasses voting and routes directly to the assigned director for approval.</p>
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

<Modal bind:open={conclusionApproveOpen} title="Approve / Reject Task Conclusion">
  <div class="form">
    {#if conclusionApproveTask}
      <p class="info-text">Task: <strong>{conclusionApproveTask.title}</strong></p>
      {#if conclusionApproveTask.final_notes}
        <div class="info-block">
          <p class="info-text"><strong>Final Notes:</strong> {conclusionApproveTask.final_notes}</p>
          {#if conclusionApproveTask.final_findings}<p class="info-text"><strong>Findings:</strong> {conclusionApproveTask.final_findings}</p>{/if}
          {#if conclusionApproveTask.key_results}<p class="info-text"><strong>Key Results:</strong> {conclusionApproveTask.key_results}</p>{/if}
          {#if conclusionApproveTask.recommendations}<p class="info-text"><strong>Recommendations:</strong> {conclusionApproveTask.recommendations}</p>{/if}
          {#if conclusionApproveTask.limitations}<p class="info-text"><strong>Limitations:</strong> {conclusionApproveTask.limitations}</p>{/if}
        </div>
      {/if}
    {/if}
    <Field label="Decision" type="select" bind:value={conclusionDecision} options={conclusionDecisionOpts} />
    <Field label="Review Notes (optional)" type="textarea" bind:value={conclusionReviewNotes} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => conclusionApproveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionApproval}>Submit Decision</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpReqOpen} title="Send Help Request">
  <div class="form">
    <p class="vote-system-note">🔀 Help requests from aerospace engineers are automatically routed to <strong>The Artificer</strong> as proxy.</p>
    <Field label="Title" bind:value={hrTitle} required />
    <Field label="Description" type="textarea" bind:value={hrDesc} rows={4} />
    <Field label="Category (optional)" bind:value={hrCategory} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpRequest}>Send Help Request</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpResolveOpen} title="Mark Help Request — In Review / Close">
  <div class="form">
    {#if helpResolveTarget}
      <p class="info-text">Request: <strong>{helpResolveTarget.title}</strong></p>
      {#if helpResolveTarget.description}
        <p class="info-text">{helpResolveTarget.description}</p>
      {/if}
    {/if}
    <Field label="Status" type="select" bind:value={helpResolveStatus} options={helpResolveOpts} />
    <Field label="Notes (optional)" type="textarea" bind:value={helpResolveResponse} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpResolveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpResolve}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpRejectOpen} title="Reject Help Request">
  <div class="form">
    {#if helpRejectTarget}
      <p class="info-text">Request: <strong>{helpRejectTarget.title}</strong></p>
      {#if helpRejectTarget.description}
        <p class="info-text">{helpRejectTarget.description}</p>
      {/if}
    {/if}
    <p class="vote-system-note">⚠️ The rejection reason will be immediately visible to the original requester. This action cannot be undone.</p>
    <Field label="Rejection Reason" type="textarea" bind:value={helpRejectReason} rows={4} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpRejectOpen = false}>Cancel</button>
      <button class="btn-danger" onclick={submitHelpReject} disabled={!helpRejectReason.trim()}>Reject Request</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpApproveOpen} title="Approve Help Request — Assign Task">
  <div class="form">
    {#if helpApproveTarget}
      <p class="info-text">Request: <strong>{helpApproveTarget.title}</strong></p>
      {#if helpApproveTarget.description}
        <p class="info-text">{helpApproveTarget.description}</p>
      {/if}
    {/if}
    <p class="vote-system-note">✅ Approving will create an assigned task in Aerospace Tasks for the selected subordinate and mark this request as <strong>converted</strong>.</p>
    <div class="field">
      <label class="field-label">Assign Task To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={helpApproveAssignee} />
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpApproveOpen = false}>Cancel</button>
      <button class="btn-approve" onclick={submitHelpApprove} disabled={!helpApproveAssignee}>Approve & Create Task</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpDeliverOpen} title="Deliver Task Response">
  <div class="form">
    {#if helpDeliverTarget}
      <p class="info-text">Request: <strong>{helpDeliverTarget.title}</strong></p>
      {#if helpDeliverTarget.created_task_id}
        <p class="info-text">Linked Task ID: <code>{helpDeliverTarget.created_task_id}</code></p>
      {/if}
    {/if}
    <p class="vote-system-note">📤 As proxy director, you are officially delivering the completed task result back to the original requester. The request will be marked <strong>resolved</strong>.</p>
    <Field label="Response / Delivery Notes" type="textarea" bind:value={helpDeliverResponse} rows={4} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpDeliverOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpDeliver} disabled={!helpDeliverResponse.trim()}>Deliver Response</button>
    </div>
  </div>
</Modal>

<Modal bind:open={shipDetailOpen} title="Ship Details">
  <div class="form">
    {#if shipDetailData}
      <div class="info-block">
        <p class="info-text"><strong>Name:</strong> {shipDetailData.ship?.ship_name ?? shipDetailData.ship?.name}</p>
        <p class="info-text"><strong>Type:</strong> {shipDetailData.ship?.ship_type ?? shipDetailData.ship?.model ?? '—'}</p>
        <p class="info-text"><strong>Status:</strong> <span class={statusBadgeClass(shipDetailData.ship?.status)}>{shipDetailData.ship?.status ?? '—'}</span></p>
        {#if shipDetailData.ship?.build_status}
          <p class="info-text"><strong>Build Status:</strong> <span class={statusBadgeClass(shipDetailData.ship?.build_status)}>{shipDetailData.ship?.build_status}</span></p>
        {/if}
        {#if shipDetailData.ship?.capacity}
          <p class="info-text"><strong>Capacity:</strong> {shipDetailData.ship.capacity}</p>
        {/if}
        {#if shipDetailData.ship?.materials_used}
          <p class="info-text"><strong>Materials Used:</strong> {shipDetailData.ship.materials_used}</p>
        {/if}
        {#if shipDetailData.ship?.launch_date}
          <p class="info-text"><strong>Launch Date:</strong> {new Date(shipDetailData.ship.launch_date).toLocaleDateString()}</p>
        {/if}
      </div>
      {#if shipDetailData.blueprints?.length > 0}
        <h3 class="section-title" style="margin-top:1rem">Blueprint Proposals</h3>
        {#each shipDetailData.blueprints as bp}
          <div class="info-block">
            <p class="info-text"><strong>{bp.ship_name}</strong> — <span class={statusBadgeClass(bp.status)}>{bp.status}</span></p>
            <p class="info-text">{bp.blueprint_description}</p>
          </div>
        {/each}
      {/if}
    {/if}
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => shipDetailOpen = false}>Close</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; margin-bottom: 0.25rem; }
  .info-block { background: rgba(61,127,255,0.06); border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.75rem 1rem; margin-bottom: 0.5rem; display: flex; flex-direction: column; gap: 0.25rem; }
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
  .desc-cell { max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-cancelled { background: rgba(255,68,102,0.12); color: #ff4466; }
  .reports-list { display: flex; flex-direction: column; gap: 1rem; }
  .report-card { background: #0a1020; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem 1.25rem; }
  .report-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem; }
  .report-title { color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; font-weight: 600; }
  .report-date { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; }
  .report-body { color: #8fa3cc; font-size: 0.85rem; line-height: 1.5; margin-bottom: 0.5rem; }
  .report-section { color: #8fa3cc; font-size: 0.82rem; line-height: 1.4; margin-top: 0.4rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .report-label { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.06em; }
  .bypass-info-row { display: flex; align-items: center; gap: 0.5rem; }
  .vote-type-badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-vote,
  .vote-type-blueprint { background: rgba(156,39,176,0.15); color: #ce93d8; }
  .vote-badge { font-family: 'Space Mono', monospace; font-size: 0.65rem; }
  .vote-badge-linked { color: #ce93d8; }
  .vote-system-note { color: #8fa3cc; font-size: 0.82rem; line-height: 1.5; margin-bottom: 1rem; background: rgba(61,127,255,0.05); border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.6rem 0.75rem; }
  .review-notes { cursor: help; }
  .btn-conclude { border-color: #ce93d8; color: #ce93d8; }
  .btn-conclude:hover { background: rgba(156,39,176,0.12); }
  .btn-approve { border-color: #00c853; color: #00c853; }
  .btn-approve:hover { background: rgba(0,200,83,0.12); }
  .btn-reject { border-color: #ff4466; color: #ff4466; }
  .btn-reject:hover { background: rgba(255,68,102,0.12); }
  .btn-danger { background: rgba(255,68,102,0.15); border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-danger:hover { background: rgba(255,68,102,0.25); }
  .btn-danger:disabled { opacity: 0.45; cursor: not-allowed; }
  .actions-cell { display: flex; gap: 0.375rem; flex-wrap: wrap; align-items: center; }
  .filter-row { display: flex; gap: 0.5rem; align-items: center; }
  .filter-select { background: #0d1a2e; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; font-family: 'Space Mono', monospace; font-size: 0.7rem; padding: 0.35rem 0.625rem; }
  .proxy-badge { background: rgba(0,212,255,0.1); border-radius: 3px; color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.65rem; padding: 0.2rem 0.5rem; }
</style>
