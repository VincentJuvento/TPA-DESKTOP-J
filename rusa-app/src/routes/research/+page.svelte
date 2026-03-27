<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { researchApi, researchTaskApi, userApi, aerospaceApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'experiments' | 'species' | 'tests' | 'research_tasks' | 'pending_conclusions' | 'observer_dashboard' | 'help_requests';
  let activeTab = $state<Tab>('experiments');

  let experiments: any[] = $state([]);
  let species: any[] = $state([]);
  let tests: any[] = $state([]);
  let researchTasks: any[] = $state([]);
  let loading = $state(false);

  // Experiment form
  let expOpen = $state(false);
  let expTitle = $state('');
  let expDesc = $state('');
  let expType = $state('');
  let expStart = $state('');
  let expEnd = $state('');

  // Species form
  let speciesOpen = $state(false);
  let speciesName = $state('');
  let speciesClass = $state('');
  let speciesDesc = $state('');
  let speciesHabitat = $state('');
  let speciesCategory = $state('unknown');

  // Help request form (research/biological/agricultural engineers)
  let helpReqOpen = $state(false);
  let hrTitle = $state('');
  let hrDesc = $state('');
  let hrCategory = $state('');
  let helpRequests: any[] = $state([]);
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

  // Test form
  let testOpen = $state(false);
  let testTitle = $state('');
  let testDesc = $state('');
  let testMethod = $state('');

  // Review form
  let reviewOpen = $state(false);
  let reviewTarget: any = $state(null);
  let reviewType = $state<'experiment' | 'test'>('experiment');
  let reviewStatus = $state('');
  let reviewNotes = $state('');

  // Assign research task form
  let assignTaskOpen = $state(false);
  let rtTitle = $state('');
  let rtDesc = $state('');
  let rtDue = $state('');

  let allUsers: any[] = $state([]);
  let assigneeSelected: any = $state(null);

  // Submit result form
  let resultOpen = $state(false);
  let selectedTask: any = $state(null);
  let resultNotes = $state('');

  // Conclusion request modal (for scientists/engineers)
  let conclusionReqOpen = $state(false);
  let conclusionReqExp: any = $state(null);
  let conclusionFinalNotes = $state('');
  let conclusionFinalFindings = $state('');
  let conclusionMethodology = $state('');
  let conclusionKeyResults = $state('');
  let conclusionRecommendations = $state('');
  let conclusionLimitations = $state('');

  // Conclusion review modal (for Taskmaster)
  let conclusionReviewOpen = $state(false);
  let conclusionReviewTarget: any = $state(null);
  let conclusionReviewLogs: any[] = $state([]);
  let conclusionDecision = $state('');
  let conclusionReviewNotes = $state('');
  let conclusionReviewLoading = $state(false);

  // Add Log modal
  let logOpen = $state(false);
  let logExp: any = $state(null);
  let logDate = $state('');
  let logPersonnel = $state('');
  let logSpecies = $state('');
  let logTestsText = $state('');
  let logLinkedTestIds: string[] = $state([]);
  let logNotes = $state('');
  let logNewSpecies = $state(false);
  let logNewSpeciesName = $state('');
  let logNewSpeciesClass = $state('');
  let logNewSpeciesHabitat = $state('');
  let logNewSpeciesDesc = $state('');

  // View Logs modal
  let viewLogsOpen = $state(false);
  let viewLogsExp: any = $state(null);
  let viewLogsData: any[] = $state([]);
  let viewLogsLoading = $state(false);

  // Observer: Assign Experiment Task modal
  let assignExpTaskOpen = $state(false);
  let assignExpTaskExp: any = $state(null);
  let assignExpTaskAssignee: any = $state(null);
  let assignExpTaskTitle = $state('');
  let assignExpTaskDue = $state('');

  // Observer dashboard data
  let observerDashboard: any = $state(null);
  let observerDashboardLoading = $state(false);

  const canAssignResearchTasks = $derived(
    canPerform($session, 'the_observer') ||
    canPerform($session, 'the_artificer') ||
    canPerform($session, 'the_taskmaster')
  );

  const isTaskmaster = $derived(canPerform($session, 'the_taskmaster'));
  const isObserver = $derived(canPerform($session, 'the_observer'));
  const isResearchEngineer = $derived(
    canPerform($session, 'biological_engineer') ||
    canPerform($session, 'agricultural_engineer')
  );
  const isAgriculturalEngineer = $derived(canPerform($session, 'agricultural_engineer'));

  const canReviewExperiment = $derived(
    $session?.role_name === 'the_observer' ||
    $session?.role_name === 'the_taskmaster' ||
    ($session?.tier ?? 0) >= 3
  );

  const pendingConclusions = $derived(experiments.filter((e: any) => e.status === 'conclusion_requested'));

  function getUserName(userId: string | undefined | null): string {
    if (!userId) return '—';
    const u = allUsers.find((u: any) => u.id === userId);
    return u ? (u.full_name ?? u.username ?? userId) : userId;
  }

  const isAuthorizedForConclusion = $derived(
    conclusionReviewTarget
      ? conclusionReviewTarget.reviewed_by === $session?.user_id
      : false
  );

  const approvedTests = $derived(tests.filter((t: any) => t.status === 'approved'));

  onMount(async () => {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      [experiments, species, tests, researchTasks, helpRequests] = await Promise.all([
        researchApi.getExperiments(s.token),
        researchApi.getSpeciesArchive(s.token),
        researchApi.getTestArchive(s.token),
        researchTaskApi.getTasks(s.token),
        aerospaceApi.getHelpRequests(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function proposeExperiment() {
    const s = $session; if (!s) return;
    if (!expTitle || !expDesc || !expType) { showToast('Title, description, and type required', 'error'); return; }
    try {
      await researchApi.proposeExperiment(s.token, expTitle, expDesc, expType, expStart ? expStart + 'T00:00:00Z' : undefined, expEnd ? expEnd + 'T00:00:00Z' : undefined);
      showToast('Experiment proposed', 'success');
      expOpen = false; expTitle = ''; expDesc = ''; expType = ''; expStart = ''; expEnd = '';
      experiments = await researchApi.getExperiments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addSpecies() {
    const s = $session; if (!s) return;
    if (!speciesName) { showToast('Name required', 'error'); return; }
    try {
      await researchApi.addSpecies(s.token, speciesName, speciesClass || undefined, speciesDesc || undefined, speciesHabitat || undefined, speciesCategory || undefined);
      showToast('Species added', 'success');
      speciesOpen = false; speciesName = ''; speciesClass = ''; speciesDesc = ''; speciesHabitat = ''; speciesCategory = 'unknown';
      species = await researchApi.getSpeciesArchive(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function proposeTest() {
    const s = $session; if (!s) return;
    if (!testTitle) { showToast('Title required', 'error'); return; }
    try {
      await researchApi.proposeTest(s.token, testTitle, testDesc || undefined, testMethod || undefined);
      showToast('Test proposed', 'success');
      testOpen = false; testTitle = ''; testDesc = ''; testMethod = '';
      tests = await researchApi.getTestArchive(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openReview(item: any, type: 'experiment' | 'test') {
    reviewTarget = item; reviewType = type; reviewStatus = ''; reviewNotes = ''; reviewOpen = true;
  }

  async function submitReview() {
    const s = $session; if (!s || !reviewTarget) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      if (reviewType === 'experiment') {
        await researchApi.reviewExperiment(s.token, reviewTarget.id, reviewStatus, reviewNotes || undefined);
        experiments = await researchApi.getExperiments(s.token);
      } else {
        await researchApi.reviewTest(s.token, reviewTarget.id, reviewStatus, reviewNotes || undefined);
        tests = await researchApi.getTestArchive(s.token);
      }
      showToast('Review submitted', 'success'); reviewOpen = false;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function assignResearchTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !rtTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await researchTaskApi.assign(s.token, rtTitle, rtDesc || undefined, assigneeSelected.id, undefined, rtDue || undefined);
      showToast('Research task assigned', 'success');
      assignTaskOpen = false; assigneeSelected = null; rtTitle = ''; rtDesc = ''; rtDue = '';
      researchTasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openResultModal(task: any) {
    selectedTask = task;
    resultNotes = task.result_notes ?? '';
    resultOpen = true;
  }

  async function submitResult() {
    const s = $session; if (!s || !selectedTask) return;
    if (!resultNotes) { showToast('Result notes required', 'error'); return; }
    try {
      await researchTaskApi.submitResult(s.token, selectedTask.id, resultNotes);
      showToast('Result submitted', 'success');
      resultOpen = false;
      researchTasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function completeTask(task: any) {
    const s = $session; if (!s) return;
    try {
      await researchTaskApi.complete(s.token, task.id);
      showToast('Task marked complete', 'success');
      researchTasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openConclusionRequest(exp: any) {
    conclusionReqExp = exp;
    conclusionFinalNotes = exp.final_notes ?? '';
    conclusionFinalFindings = exp.final_findings ?? '';
    conclusionMethodology = exp.methodology_summary ?? '';
    conclusionKeyResults = exp.key_results ?? '';
    conclusionRecommendations = exp.recommendations ?? '';
    conclusionLimitations = exp.limitations ?? '';
    conclusionReqOpen = true;
  }

  async function submitConclusionRequest() {
    const s = $session; if (!s || !conclusionReqExp) return;
    if (!conclusionFinalNotes.trim()) { showToast('Final summary is required', 'error'); return; }
    if (!conclusionMethodology.trim()) { showToast('Methodology summary is required', 'error'); return; }
    if (!conclusionKeyResults.trim()) { showToast('Key results are required', 'error'); return; }
    try {
      await researchApi.requestConclusion(
        s.token,
        conclusionReqExp.id,
        conclusionFinalNotes,
        conclusionFinalFindings || undefined,
        conclusionMethodology || undefined,
        conclusionKeyResults || undefined,
        conclusionRecommendations || undefined,
        conclusionLimitations || undefined,
      );
      showToast('Conclusion request submitted', 'success');
      conclusionReqOpen = false;
      experiments = await researchApi.getExperiments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openConclusionReview(exp: any) {
    const s = $session; if (!s) return;
    conclusionReviewTarget = exp;
    conclusionDecision = '';
    conclusionReviewNotes = '';
    conclusionReviewLoading = true;
    conclusionReviewOpen = true;
    try {
      conclusionReviewLogs = await researchApi.getExperimentLogs(s.token, exp.id);
    } catch (_e: any) {
      conclusionReviewLogs = [];
    }
    conclusionReviewLoading = false;
  }

  async function submitConclusionReview() {
    const s = $session; if (!s || !conclusionReviewTarget) return;
    if (!conclusionDecision) { showToast('Decision required', 'error'); return; }
    try {
      await researchApi.approveConclusion(s.token, conclusionReviewTarget.id, conclusionDecision, conclusionReviewNotes || undefined);
      showToast(conclusionDecision === 'approve' ? 'Conclusion approved' : 'Conclusion rejected', 'success');
      conclusionReviewOpen = false;
      experiments = await researchApi.getExperiments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openAddLog(exp: any) {
    logExp = exp;
    logDate = new Date().toISOString().slice(0, 16);
    logPersonnel = '';
    logSpecies = '';
    logTestsText = '';
    logLinkedTestIds = [];
    logNotes = '';
    logNewSpecies = false;
    logNewSpeciesName = '';
    logNewSpeciesClass = '';
    logNewSpeciesHabitat = '';
    logNewSpeciesDesc = '';
    logOpen = true;
  }

  function toggleTestLink(testId: string) {
    if (logLinkedTestIds.includes(testId)) {
      logLinkedTestIds = logLinkedTestIds.filter(id => id !== testId);
    } else {
      logLinkedTestIds = [...logLinkedTestIds, testId];
    }
  }

  async function submitLog() {
    const s = $session; if (!s || !logExp) return;
    if (!logDate) { showToast('Log date is required', 'error'); return; }
    try {
      const dateVal = logDate.includes('T') ? new Date(logDate).toISOString() : new Date(logDate + 'T00:00:00Z').toISOString();
      const linkedJson = logLinkedTestIds.length > 0 ? JSON.stringify(logLinkedTestIds) : undefined;
      const logId = await researchApi.addLog(
        s.token, logExp.id, dateVal,
        logPersonnel || undefined,
        logSpecies || undefined,
        logTestsText || undefined,
        linkedJson,
        logNotes || undefined,
      );

      // If new species discovered, create proposal
      if (logNewSpecies && logNewSpeciesName.trim()) {
        try {
          await researchApi.proposeSpeciesFromDiscovery(
            s.token, logExp.id, logNewSpeciesName,
            logNewSpeciesDesc || undefined,
            logNewSpeciesClass || undefined,
            logNewSpeciesHabitat || undefined,
          );
          showToast('New species discovery proposed for archive', 'success');
          species = await researchApi.getSpeciesArchive(s.token);
        } catch (se: any) {
          showToast('Log saved but species proposal failed: ' + se, 'error');
        }
      }

      showToast('Log entry added', 'success');
      logOpen = false;
      experiments = await researchApi.getExperiments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openViewLogs(exp: any) {
    const s = $session; if (!s) return;
    viewLogsExp = exp;
    viewLogsData = [];
    viewLogsLoading = true;
    viewLogsOpen = true;
    try {
      viewLogsData = await researchApi.getExperimentLogs(s.token, exp.id);
    } catch (_e: any) {
      viewLogsData = [];
    }
    viewLogsLoading = false;
  }

  function getLinkedTestNames(linkedTestIds: string | null): string {
    if (!linkedTestIds) return '';
    try {
      const ids: string[] = JSON.parse(linkedTestIds);
      if (!Array.isArray(ids)) return linkedTestIds;
      return ids.map(id => {
        const t = tests.find((t: any) => t.id === id);
        return t ? t.title : id.slice(0, 8) + '…';
      }).join(', ');
    } catch {
      return linkedTestIds;
    }
  }

  function openAssignExpTask(exp: any) {
    assignExpTaskExp = exp;
    assignExpTaskAssignee = null;
    assignExpTaskTitle = '';
    assignExpTaskDue = '';
    assignExpTaskOpen = true;
  }

  async function submitAssignExpTask() {
    const s = $session; if (!s || !assignExpTaskExp) return;
    if (!assignExpTaskAssignee || !assignExpTaskTitle.trim()) { showToast('Assignee and title required', 'error'); return; }
    try {
      await researchApi.assignExperimentTask(
        s.token,
        assignExpTaskExp.id,
        assignExpTaskAssignee.id,
        assignExpTaskTitle,
        assignExpTaskDue ? new Date(assignExpTaskDue).toISOString() : undefined,
      );
      showToast('Task assigned', 'success');
      assignExpTaskOpen = false;
      if (observerDashboard) await loadObserverDashboard();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function loadObserverDashboard() {
    const s = $session; if (!s) return;
    observerDashboardLoading = true;
    try {
      observerDashboard = await researchApi.getObserverDashboard(s.token);
    } catch (e: any) { showToast('Failed to load dashboard: ' + e, 'error'); }
    observerDashboardLoading = false;
  }

  $effect(() => {
    if (activeTab === 'observer_dashboard' && isObserver && !observerDashboard) {
      loadObserverDashboard();
    }
  });

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
      case 'pending': return 'badge badge-open';
      case 'result_submitted': return 'badge badge-progress';
      case 'completed': return 'badge badge-done';
      default: return 'badge';
    }
  }

  function expStatusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'pending': return 'badge-open';
      case 'approved': return 'badge-approved';
      case 'in_progress': return 'badge-progress';
      case 'conclusion_requested': return 'badge-conclude';
      case 'completed': return 'badge-done';
      case 'rejected': return 'badge-rejected';
      case 'cancelled': return 'badge-rejected';
      default: return '';
    }
  }

  const expCols = [
    { key: 'title', label: 'Title' },
    { key: 'experiment_type', label: 'Type' },
    { key: 'status', label: 'Status' },
    { key: 'start_date', label: 'Start Date' },
  ];
  const speciesCols = [
    { key: 'name', label: 'Name' },
    { key: 'classification', label: 'Classification' },
    { key: 'habitat', label: 'Habitat' },
    { key: 'approval_status', label: 'Status' },
  ];
  const testCols = [
    { key: 'title', label: 'Title' },
    { key: 'status', label: 'Status' },
    { key: 'methodology', label: 'Methodology' },
  ];
  const reviewStatusOpts = [
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
    { value: 'under_review', label: 'Under Review' },
  ];
  const helpResolveOpts = [
    { value: 'in_review', label: 'In Review' },
    { value: 'closed', label: 'Closed' },
  ];
  const speciesCategoryOpts = [
    { value: 'unknown', label: 'Unknown' },
    { value: 'plant', label: 'Plant' },
    { value: 'animal', label: 'Animal' },
    { value: 'microorganism', label: 'Microorganism' },
    { value: 'fungus', label: 'Fungus' },
    { value: 'other', label: 'Other' },
  ];
  const isDirector = $derived(($session?.tier ?? 0) >= 3);
</script>

<svelte:head><title>RUSA IMS — Research & Lab</title></svelte:head>

<PageShell title="Research & Lab" subtitle="Experiments, species archive, test proposals, and research tasks">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='experiments'} onclick={() => activeTab='experiments'}>Experiments</button>
    <button class="tab" class:active={activeTab==='species'} onclick={() => activeTab='species'}>Species Archive</button>
    <button class="tab" class:active={activeTab==='tests'} onclick={() => activeTab='tests'}>Test Archive</button>
    <button class="tab" class:active={activeTab==='research_tasks'} onclick={() => activeTab='research_tasks'}>
      {canAssignResearchTasks ? 'Research Tasks' : 'My Research Tasks'}
    </button>
    {#if isTaskmaster}
      <button class="tab" class:active={activeTab==='pending_conclusions'} onclick={() => activeTab='pending_conclusions'}>
        Pending Conclusions {#if pendingConclusions.length > 0}<span class="tab-badge">{pendingConclusions.length}</span>{/if}
      </button>
    {/if}
    {#if isObserver}
      <button class="tab" class:active={activeTab==='observer_dashboard'} onclick={() => activeTab='observer_dashboard'}>
        Observer Dashboard
      </button>
    {/if}
    <button class="tab" class:active={activeTab==='help_requests'} onclick={() => activeTab='help_requests'}>
      Help Requests {#if helpRequests.filter((r: any) => r.status === 'open').length > 0}<span class="tab-badge">{helpRequests.filter((r: any) => r.status === 'open').length}</span>{/if}
    </button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'experiments'}
    <div class="section-bar">
      <h2 class="section-title">Experiments</h2>
      <button class="btn-primary" onclick={() => expOpen = true}>+ Propose Experiment</button>
    </div>
    {#if experiments.length === 0}
      <p class="empty">No experiments yet.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Type</th>
              <th>Status</th>
              <th>Start Date</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each experiments as exp}
              <tr>
                <td>{exp.title}</td>
                <td>{exp.experiment_type ?? '—'}</td>
                <td>
                  <span class="badge {expStatusBadgeClass(exp.status)}">{exp.status ?? '—'}</span>
                  {#if exp.conclusion_approved}
                    <span class="badge badge-done" style="margin-left:4px">Concluded</span>
                  {/if}
                </td>
                <td>{exp.start_date ? new Date(exp.start_date).toLocaleDateString() : '—'}</td>
                <td class="actions-cell">
                  <button class="btn-small" onclick={() => openViewLogs(exp)}>View Logs</button>
                  {#if exp.status === 'in_progress' || exp.status === 'approved'}
                    <button class="btn-small btn-add" onclick={() => openAddLog(exp)}>+ Log</button>
                  {/if}
                  {#if canReviewExperiment}
                    <button class="btn-small" onclick={() => openReview(exp, 'experiment')}>Review</button>
                  {/if}
                  {#if isObserver}
                    <button class="btn-small btn-observer" onclick={() => openAssignExpTask(exp)}>Assign Task</button>
                  {/if}
                  {#if exp.status === 'in_progress' && exp.proposed_by === $session?.user_id}
                    <button class="btn-small btn-conclude" onclick={() => openConclusionRequest(exp)}>Request Conclusion</button>
                  {/if}
                  {#if isTaskmaster && exp.status === 'conclusion_requested'}
                    <button class="btn-small btn-conclude" onclick={() => openConclusionReview(exp)}>Review Conclusion</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'species'}
    <div class="section-bar">
      <h2 class="section-title">
        Species Archive
        {#if isAgriculturalEngineer}
          <span class="plant-only-badge">🌿 Plants Only</span>
        {/if}
      </h2>
      <button class="btn-primary" onclick={() => { speciesCategory = isAgriculturalEngineer ? 'plant' : 'unknown'; speciesOpen = true; }}>+ Add Species</button>
    </div>
    {#if isAgriculturalEngineer}
      <p class="access-note">⚠️ Agricultural engineers have access to <strong>plant species only</strong>. New species are pre-categorized as plants.</p>
    {/if}
    {#if species.length === 0}
      <p class="empty">No species in archive.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Category</th>
              <th>Classification</th>
              <th>Habitat</th>
              <th>Archive Status</th>
              <th>Discovery Experiment</th>
            </tr>
          </thead>
          <tbody>
            {#each species as sp}
              <tr>
                <td>{sp.name}</td>
                <td>
                  {#if sp.species_category && sp.species_category !== 'unknown'}
                    <span class="species-cat-badge species-cat-{sp.species_category}">{sp.species_category}</span>
                  {:else}
                    <span class="badge">—</span>
                  {/if}
                </td>
                <td>{sp.classification ?? '—'}</td>
                <td>{sp.habitat ?? '—'}</td>
                <td>
                  {#if sp.approval_status === 'pending_approval'}
                    <span class="badge badge-conclude">Pending Approval</span>
                  {:else}
                    <span class="badge badge-done">{sp.approval_status ?? 'Approved'}</span>
                  {/if}
                </td>
                <td>
                  {#if sp.discovery_experiment_id}
                    {experiments.find((e: any) => e.id === sp.discovery_experiment_id)?.title ?? sp.discovery_experiment_id.slice(0,8) + '…'}
                  {:else}
                    —
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'tests'}
    <div class="section-bar">
      <h2 class="section-title">Test Archive</h2>
      <button class="btn-primary" onclick={() => testOpen = true}>+ Propose Test</button>
    </div>
    <Table columns={testCols} rows={tests} onRowClick={(r) => openReview(r, 'test')} />
  {:else if activeTab === 'research_tasks'}
    <div class="section-bar">
      <h2 class="section-title">{canAssignResearchTasks ? 'Tasks I Assigned' : 'My Research Tasks'}</h2>
      {#if canAssignResearchTasks}
        <button class="btn-primary" onclick={() => assignTaskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if researchTasks.length === 0}
      <p class="empty">{canAssignResearchTasks ? 'No research tasks assigned yet.' : 'No research tasks have been assigned to you.'}</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Result Notes</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each researchTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>{task.result_notes ?? '—'}</td>
                <td>
                  {#if !canAssignResearchTasks && task.status === 'pending'}
                    <button class="btn-small" onclick={() => openResultModal(task)}>Submit Result</button>
                  {/if}
                  {#if canAssignResearchTasks && task.status === 'result_submitted'}
                    <button class="btn-small" onclick={() => completeTask(task)}>Mark Complete</button>
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
      <h2 class="section-title">Pending Experiment Conclusions</h2>
    </div>
    {#if pendingConclusions.length === 0}
      <p class="empty">No experiment conclusions pending approval.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Experiment Title</th>
              <th>Type</th>
              <th>Requested At</th>
              <th>Original Approver</th>
              <th>Final Notes (Preview)</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each pendingConclusions as exp}
              <tr>
                <td>{exp.title}</td>
                <td>{exp.experiment_type ?? '—'}</td>
                <td>{exp.conclusion_requested_at ? new Date(exp.conclusion_requested_at).toLocaleString() : '—'}</td>
                <td>{getUserName(exp.reviewed_by)}</td>
                <td class="notes-preview">{exp.final_notes ? exp.final_notes.slice(0, 80) + (exp.final_notes.length > 80 ? '…' : '') : '—'}</td>
                <td>
                  <button class="btn-small btn-conclude" onclick={() => openConclusionReview(exp)}>Review</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'observer_dashboard'}
    <div class="section-bar">
      <h2 class="section-title">Observer Dashboard</h2>
      <button class="btn-secondary" onclick={loadObserverDashboard}>↻ Refresh</button>
    </div>
    {#if observerDashboardLoading}
      <p class="loading">Loading dashboard…</p>
    {:else if observerDashboard}
      <div class="dashboard-section">
        <h3 class="subsection-title">Active Experiments</h3>
        {#if observerDashboard.active_experiments?.length === 0}
          <p class="empty">No active experiments.</p>
        {:else}
          <div class="table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Title</th>
                  <th>Type</th>
                  <th>Status</th>
                  <th>Days Elapsed</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each observerDashboard.active_experiments as exp}
                  <tr>
                    <td>{exp.title}</td>
                    <td>{exp.experiment_type ?? '—'}</td>
                    <td><span class="badge {expStatusBadgeClass(exp.status)}">{exp.status ?? '—'}</span></td>
                    <td>{exp.days_elapsed ?? 0} days</td>
                    <td>
                      <button class="btn-small btn-observer" onclick={() => openAssignExpTask(exp)}>+ Assign Task</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
      <div class="dashboard-section">
        <h3 class="subsection-title">Tasks I've Assigned</h3>
        {#if observerDashboard.assigned_tasks?.length === 0}
          <p class="empty">No tasks assigned yet.</p>
        {:else}
          <div class="table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Task Title</th>
                  <th>Status</th>
                  <th>Due Date</th>
                  <th>Experiment</th>
                </tr>
              </thead>
              <tbody>
                {#each observerDashboard.assigned_tasks as task}
                  <tr>
                    <td>{task.title}</td>
                    <td>
                      <span class="badge {task.status === 'completed' ? 'badge-done' : task.status === 'in_progress' ? 'badge-progress' : 'badge-open'}">{task.status}</span>
                      {#if task.due_date && new Date(task.due_date) < new Date() && task.status !== 'completed'}
                        <span class="badge badge-rejected" style="margin-left:4px">Overdue</span>
                      {/if}
                    </td>
                    <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                    <td>{observerDashboard.active_experiments?.find((e: any) => e.id === task.experiment_id)?.title ?? '—'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {:else}
      <p class="empty">Dashboard not loaded.</p>
    {/if}

  {:else if activeTab === 'help_requests'}
    <div class="section-bar">
      <h2 class="section-title">Help Requests</h2>
      {#if isResearchEngineer}
        <button class="btn-primary" onclick={() => helpReqOpen = true}>+ Send Help Request</button>
      {/if}
    </div>
    {#if isResearchEngineer}
      <p class="access-note">🔀 Help requests are automatically routed to <strong>The Observer</strong> as proxy director.</p>
    {/if}
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
                <td><span class="badge {req.status === 'resolved' || req.status === 'closed' ? 'badge-done' : req.status === 'rejected' ? 'badge-rejected' : req.status === 'converted' ? 'badge-progress' : 'badge-open'}">{req.status ?? '—'}</span></td>
                <td class="notes-preview">
                  {#if req.status === 'rejected' && req.rejection_reason}
                    <span class="badge badge-rejected" style="font-size:0.7rem">Rejected:</span> {req.rejection_reason}
                  {:else}
                    {req.response ?? '—'}
                  {/if}
                </td>
                <td>{req.created_at ? new Date(req.created_at).toLocaleDateString() : '—'}</td>
                {#if isDirector}
                  <td>
                    <div class="actions-cell">
                      {#if req.status === 'open' || req.status === 'in_review'}
                        <button class="btn-small btn-add" onclick={() => openHelpApprove(req)}>Approve</button>
                        <button class="btn-small btn-danger-sm" onclick={() => openHelpReject(req)}>Reject</button>
                        <button class="btn-small btn-observer" onclick={() => openHelpResolve(req)}>Mark Review</button>
                      {:else if req.status === 'converted'}
                        <button class="btn-small btn-observer" onclick={() => openHelpDeliver(req)}>Deliver Response</button>
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

<Modal bind:open={expOpen} title="Propose Experiment">
  <div class="form">
    <Field label="Title" bind:value={expTitle} required />
    <Field label="Description" type="textarea" bind:value={expDesc} required />
    <Field label="Experiment Type" bind:value={expType} placeholder="e.g. biology, chemistry" required />
    <Field label="Start Date" type="date" bind:value={expStart} />
    <Field label="End Date" type="date" bind:value={expEnd} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => expOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={proposeExperiment}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={speciesOpen} title="Add Species">
  <div class="form">
    <Field label="Name" bind:value={speciesName} required />
    <Field label="Category" type="select" bind:value={speciesCategory} options={speciesCategoryOpts} />
    <Field label="Classification" bind:value={speciesClass} />
    <Field label="Description" type="textarea" bind:value={speciesDesc} />
    <Field label="Habitat" bind:value={speciesHabitat} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => speciesOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addSpecies}>Add Species</button>
    </div>
  </div>
</Modal>

<Modal bind:open={testOpen} title="Propose Test">
  <div class="form">
    <Field label="Title" bind:value={testTitle} required />
    <Field label="Description" type="textarea" bind:value={testDesc} />
    <Field label="Methodology" type="textarea" bind:value={testMethod} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => testOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={proposeTest}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={reviewOpen} title="Review {reviewTarget?.title}">
  <div class="form">
    <Field label="Status" type="select" bind:value={reviewStatus} options={reviewStatusOpts} required />
    <Field label="Notes" type="textarea" bind:value={reviewNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<Modal bind:open={assignTaskOpen} title="Assign Research Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assigneeSelected} />
    </div>
    <Field label="Title" bind:value={rtTitle} required />
    <Field label="Description" type="textarea" bind:value={rtDesc} />
    <Field label="Due Date" type="datetime-local" bind:value={rtDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { assignTaskOpen = false; assigneeSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={assignResearchTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={resultOpen} title="Submit Task Result">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedTask?.title}</strong></p>
    <Field label="Result Notes" type="textarea" bind:value={resultNotes} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => resultOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitResult}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={conclusionReqOpen} title="Request Experiment Conclusion">
  <div class="form">
    <div class="info-block">
      <p class="info-text">Experiment: <strong>{conclusionReqExp?.title}</strong></p>
      <p class="info-text">Started: <strong>{conclusionReqExp?.start_date ? new Date(conclusionReqExp.start_date).toLocaleDateString() : '—'}</strong></p>
    </div>
    <Field label="Executive Summary" type="textarea" bind:value={conclusionFinalNotes} rows={4} required hint="High-level summary of the experiment outcome." />
    <Field label="Methodology Summary" type="textarea" bind:value={conclusionMethodology} rows={3} required hint="How the experiment was conducted." />
    <Field label="Key Results / Findings" type="textarea" bind:value={conclusionKeyResults} rows={3} required hint="Main findings and results observed." />
    <Field label="Final Findings (Detail)" type="textarea" bind:value={conclusionFinalFindings} rows={3} hint="Detailed findings and statistical results (if applicable)." />
    <Field label="Recommendations" type="textarea" bind:value={conclusionRecommendations} rows={3} hint="Suggested next steps or actions." />
    <Field label="Limitations" type="textarea" bind:value={conclusionLimitations} rows={3} hint="Known limitations or caveats of the experiment." />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => conclusionReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionRequest}>Request Conclusion</button>
    </div>
  </div>
</Modal>

<Modal bind:open={conclusionReviewOpen} title="Review Experiment Conclusion">
  <div class="form">
    {#if conclusionReviewTarget}
      <div class="info-block">
        <p class="info-text">Experiment: <strong>{conclusionReviewTarget.title}</strong></p>
        <p class="info-text">Type: <strong>{conclusionReviewTarget.experiment_type ?? '—'}</strong></p>
        <p class="info-text">Requested: <strong>{conclusionReviewTarget.conclusion_requested_at ? new Date(conclusionReviewTarget.conclusion_requested_at).toLocaleString() : '—'}</strong></p>
        <p class="info-text">Original Approver: <strong>{getUserName(conclusionReviewTarget.reviewed_by)}</strong></p>
        {#if isAuthorizedForConclusion}
          <p class="auth-badge auth-ok">✓ You are authorized to approve this conclusion</p>
        {:else}
          <p class="auth-badge auth-denied">✗ Not authorized — only the original proposal approver can approve this conclusion</p>
        {/if}
      </div>
      <div class="field">
        <span class="field-label">Executive Summary</span>
        <div class="readonly-block">{conclusionReviewTarget.final_notes ?? '—'}</div>
      </div>
      {#if conclusionReviewTarget.methodology_summary}
        <div class="field">
          <span class="field-label">Methodology</span>
          <div class="readonly-block">{conclusionReviewTarget.methodology_summary}</div>
        </div>
      {/if}
      {#if conclusionReviewTarget.key_results}
        <div class="field">
          <span class="field-label">Key Results</span>
          <div class="readonly-block">{conclusionReviewTarget.key_results}</div>
        </div>
      {/if}
      {#if conclusionReviewTarget.final_findings}
        <div class="field">
          <span class="field-label">Final Findings</span>
          <div class="readonly-block">{conclusionReviewTarget.final_findings}</div>
        </div>
      {/if}
      {#if conclusionReviewTarget.recommendations}
        <div class="field">
          <span class="field-label">Recommendations</span>
          <div class="readonly-block">{conclusionReviewTarget.recommendations}</div>
        </div>
      {/if}
      {#if conclusionReviewTarget.limitations}
        <div class="field">
          <span class="field-label">Limitations</span>
          <div class="readonly-block">{conclusionReviewTarget.limitations}</div>
        </div>
      {/if}
      <div class="field">
        <span class="field-label">Experiment Logs ({conclusionReviewLoading ? '…' : conclusionReviewLogs.length})</span>
        {#if conclusionReviewLoading}
          <p class="info-text">Loading logs…</p>
        {:else if conclusionReviewLogs.length === 0}
          <p class="info-text">No logs found.</p>
        {:else}
          <div class="logs-scroll">
            {#each conclusionReviewLogs as log}
              <div class="log-entry">
                <span class="log-date">{log.log_date ?? '—'}</span>
                {#if log.linked_test_ids}
                  <p class="log-field"><strong>Tests Performed:</strong> {getLinkedTestNames(log.linked_test_ids)}</p>
                {:else if log.tests_performed}
                  <p class="log-field"><strong>Tests:</strong> {log.tests_performed}</p>
                {/if}
                {#if log.species_matter_tested}<p class="log-field"><strong>Subject:</strong> {log.species_matter_tested}</p>{/if}
                {#if log.notes}<p class="log-field">{log.notes}</p>{/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
      <Field label="Decision" type="select" bind:value={conclusionDecision} options={[{value:'approve',label:'Approve'},{value:'reject',label:'Reject'}]} required />
      <Field label="Review Notes (optional)" type="textarea" bind:value={conclusionReviewNotes} rows={3} />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => conclusionReviewOpen = false}>Cancel</button>
        <button class="btn-danger" onclick={submitConclusionReview} disabled={!conclusionDecision || !isAuthorizedForConclusion}>
          {conclusionDecision === 'approve' ? 'Approve Conclusion' : conclusionDecision === 'reject' ? 'Reject Conclusion' : 'Submit Decision'}
        </button>
      </div>
    {/if}
  </div>
</Modal>

<!-- Add Daily Log Modal -->
<Modal bind:open={logOpen} title="Add Daily Log — {logExp?.title ?? ''}">
  <div class="form">
    <Field label="Log Date" type="datetime-local" bind:value={logDate} required />
    <Field label="Personnel Present" bind:value={logPersonnel} placeholder="Names of team members attending" />
    <Field label="Species / Matter Tested" bind:value={logSpecies} placeholder="What was tested today" />

    <div class="field">
      <span class="field-label">Tests Performed (select from approved tests)</span>
      {#if approvedTests.length === 0}
        <p class="info-text">No approved tests available. Use free text below.</p>
      {:else}
        <div class="test-checklist">
          {#each approvedTests as t}
            <label class="test-check-item">
              <input type="checkbox" checked={logLinkedTestIds.includes(t.id)} onchange={() => toggleTestLink(t.id)} />
              <span class="test-check-label">{t.title}</span>
              {#if t.methodology}<span class="test-check-meta">{t.methodology.slice(0, 60)}{t.methodology.length > 60 ? '…' : ''}</span>{/if}
            </label>
          {/each}
        </div>
      {/if}
      <input class="field-input" style="margin-top:0.4rem" bind:value={logTestsText} placeholder="Or describe tests in free text…" />
    </div>

    <Field label="Notes / Progress" type="textarea" bind:value={logNotes} rows={3} />

    <div class="field">
      <label class="discovery-toggle">
        <input type="checkbox" bind:checked={logNewSpecies} />
        <span class="field-label" style="display:inline">New species/matter discovered in this log</span>
      </label>
    </div>

    {#if logNewSpecies}
      <div class="discovery-form">
        <p class="discovery-label">New Discovery Details</p>
        <Field label="Species / Matter Name" bind:value={logNewSpeciesName} required />
        <Field label="Classification" bind:value={logNewSpeciesClass} />
        <Field label="Habitat" bind:value={logNewSpeciesHabitat} />
        <Field label="Description" type="textarea" bind:value={logNewSpeciesDesc} rows={2} />
      </div>
    {/if}

    <div class="form-actions">
      <button class="btn-secondary" onclick={() => logOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitLog}>Save Log</button>
    </div>
  </div>
</Modal>

<!-- View Logs Modal -->
<Modal bind:open={viewLogsOpen} title="Logs — {viewLogsExp?.title ?? ''}">
  <div class="form">
    {#if viewLogsLoading}
      <p class="info-text">Loading logs…</p>
    {:else if viewLogsData.length === 0}
      <p class="empty">No log entries yet for this experiment.</p>
    {:else}
      <div class="logs-scroll" style="max-height:400px">
        {#each viewLogsData as log}
          <div class="log-entry">
            <div class="log-header">
              <span class="log-date">{log.log_date ?? '—'}</span>
              {#if log.personnel_present}<span class="log-personnel">👥 {log.personnel_present}</span>{/if}
            </div>
            {#if log.linked_test_ids}
              <p class="log-field"><strong>Tests Linked:</strong> <span class="test-linked">{getLinkedTestNames(log.linked_test_ids)}</span></p>
            {:else if log.tests_performed}
              <p class="log-field"><strong>Tests:</strong> {log.tests_performed}</p>
            {/if}
            {#if log.species_matter_tested}<p class="log-field"><strong>Subject:</strong> {log.species_matter_tested}</p>{/if}
            {#if log.notes}<p class="log-field">{log.notes}</p>{/if}
            {#if log.new_species_proposed}<p class="log-field discovery-badge">🔬 New species proposed to archive</p>{/if}
          </div>
        {/each}
      </div>
    {/if}
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => viewLogsOpen = false}>Close</button>
    </div>
  </div>
</Modal>

<!-- Observer: Assign Experiment Task Modal -->
<Modal bind:open={assignExpTaskOpen} title="Assign Task to Experiment">
  <div class="form">
    <div class="info-block">
      <p class="info-text">Experiment: <strong>{assignExpTaskExp?.title ?? '—'}</strong></p>
    </div>
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assignExpTaskAssignee} />
    </div>
    <Field label="Task Title" bind:value={assignExpTaskTitle} required />
    <Field label="Due Date" type="datetime-local" bind:value={assignExpTaskDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => assignExpTaskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitAssignExpTask}>Assign Task</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpReqOpen} title="Send Help Request">
  <div class="form">
    <p class="access-note">🔀 This request will be automatically routed to <strong>The Observer</strong> as proxy director.</p>
    <Field label="Title" bind:value={hrTitle} required />
    <Field label="Description" type="textarea" bind:value={hrDesc} rows={4} />
    <Field label="Category (optional)" bind:value={hrCategory} placeholder="e.g. equipment, data, guidance" />
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
    <p class="access-note">⚠️ The rejection reason will be immediately visible to the original requester. This action cannot be undone.</p>
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
    <p class="access-note" style="background:rgba(0,200,83,0.06);border-color:rgba(0,200,83,0.2);color:#00c853">✅ Approving will create an assigned research task for the selected subordinate and mark this request as <strong>converted</strong>.</p>
    <div class="field">
      <label class="field-label">Assign Task To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={helpApproveAssignee} />
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpApproveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpApprove} disabled={!helpApproveAssignee}>Approve & Create Task</button>
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
    <p class="access-note">📤 As proxy director, you are officially delivering the completed task result back to the original requester. The request will be marked <strong>resolved</strong>.</p>
    <Field label="Response / Delivery Notes" type="textarea" bind:value={helpDeliverResponse} rows={4} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpDeliverOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpDeliver} disabled={!helpDeliverResponse.trim()}>Deliver Response</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .subsection-title { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; margin: 1.5rem 0 0.75rem; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.25rem 0.625rem; }
  .btn-small:hover { border-color: #3d7fff; color: #3d7fff; }
  .btn-add { border-color: #00c853; color: #00c853; }
  .btn-add:hover { border-color: #00e676; color: #00e676; }
  .btn-observer { border-color: #f59e0b; color: #f59e0b; }
  .btn-observer:hover { border-color: #fbbf24; color: #fbbf24; }
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
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-input { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-size: 0.85rem; padding: 0.5rem 0.75rem; width: 100%; box-sizing: border-box; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .btn-conclude { border-color: #7c3aed; color: #a78bfa; }
  .btn-conclude:hover { border-color: #a78bfa; color: #c4b5fd; }
  .btn-danger { background: linear-gradient(135deg, #dc2626, #ef4444); border: none; border-radius: 4px; color: #fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-danger-sm { border-color: #ff4466; color: #ff4466; }
  .btn-danger-sm:hover { background: rgba(255,68,102,0.12); }
  .actions-cell { white-space: nowrap; display: flex; gap: 0.375rem; align-items: center; flex-wrap: wrap; }
  .notes-preview { max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #8fa3cc; font-size: 0.8rem; }
  .tab-badge { background: #7c3aed; border-radius: 10px; color: #fff; font-size: 0.6rem; font-weight: 700; margin-left: 4px; padding: 1px 6px; }
  .info-block { background: rgba(61,127,255,0.05); border: 1px solid #1e2d4a; border-radius: 4px; display: flex; flex-direction: column; gap: 0.25rem; padding: 0.75rem; }
  .readonly-block { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #c8d8f0; font-size: 0.85rem; line-height: 1.6; min-height: 60px; padding: 0.625rem 0.875rem; white-space: pre-wrap; }
  .logs-scroll { border: 1px solid #1e2d4a; border-radius: 4px; max-height: 220px; overflow-y: auto; }
  .log-entry { border-bottom: 1px solid #1e2d4a; padding: 0.5rem 0.75rem; }
  .log-entry:last-child { border-bottom: none; }
  .log-header { display: flex; gap: 1rem; align-items: baseline; margin-bottom: 0.2rem; }
  .log-date { color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.7rem; }
  .log-personnel { color: #8fa3cc; font-size: 0.75rem; }
  .log-field { color: #c8d8f0; font-size: 0.8rem; margin: 0.2rem 0 0; }
  .test-linked { color: #a78bfa; }
  .badge-approved { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-rejected { background: rgba(255,68,102,0.12); color: #ff4466; }
  .badge-conclude { background: rgba(124,58,237,0.15); color: #a78bfa; }
  .auth-badge { border-radius: 3px; font-size: 0.8rem; font-weight: 700; padding: 0.35rem 0.625rem; }
  .auth-ok { background: rgba(0,200,83,0.12); color: #00c853; }
  .auth-denied { background: rgba(255,68,102,0.12); color: #ff4466; }
  .test-checklist { border: 1px solid #1e2d4a; border-radius: 4px; max-height: 150px; overflow-y: auto; padding: 0.375rem; display: flex; flex-direction: column; gap: 0.25rem; }
  .test-check-item { display: flex; align-items: flex-start; gap: 0.5rem; cursor: pointer; padding: 0.25rem 0.375rem; border-radius: 3px; }
  .test-check-item:hover { background: rgba(61,127,255,0.06); }
  .test-check-label { color: #c8d8f0; font-size: 0.85rem; }
  .test-check-meta { color: #4a5d82; font-size: 0.75rem; margin-left: auto; }
  .discovery-toggle { display: flex; align-items: center; gap: 0.5rem; cursor: pointer; }
  .discovery-form { background: rgba(0,200,83,0.05); border: 1px solid rgba(0,200,83,0.2); border-radius: 4px; display: flex; flex-direction: column; gap: 0.75rem; padding: 0.875rem; }
  .discovery-label { color: #00c853; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; text-transform: uppercase; margin: 0; }
  .discovery-badge { background: rgba(0,200,83,0.08); border-radius: 3px; color: #00c853; display: inline-block; font-size: 0.8rem; padding: 2px 6px; }
  .dashboard-section { margin-bottom: 2rem; }
  .access-note { background: rgba(255,193,7,0.07); border: 1px solid rgba(255,193,7,0.2); border-radius: 4px; color: #ffc107; font-size: 0.82rem; line-height: 1.5; margin-bottom: 1rem; padding: 0.6rem 0.75rem; }
  .plant-only-badge { background: rgba(0,200,83,0.12); border-radius: 3px; color: #00c853; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; margin-left: 0.5rem; padding: 0.2rem 0.5rem; text-transform: uppercase; vertical-align: middle; }
  .proxy-badge { background: rgba(0,212,255,0.1); border-radius: 3px; color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.65rem; padding: 0.2rem 0.5rem; }
  .species-cat-badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .species-cat-plant { background: rgba(0,200,83,0.12); color: #00c853; }
  .species-cat-animal { background: rgba(255,152,0,0.12); color: #ff9800; }
  .species-cat-microorganism { background: rgba(156,39,176,0.12); color: #ce93d8; }
  .species-cat-fungus { background: rgba(121,85,72,0.15); color: #bcaaa4; }
  .species-cat-other { background: rgba(61,127,255,0.1); color: #8fa3cc; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; margin-bottom: 0.25rem; }
</style>
