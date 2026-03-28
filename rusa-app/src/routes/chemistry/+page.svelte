<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { chemistryApi, researchApi, aerospaceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'matter_archive' | 'experiments' | 'tests' | 'help_requests' | 'observer_dashboard';
  let activeTab = $state<Tab>('matter_archive');

  let matters: any[] = $state([]);
  let experiments: any[] = $state([]);
  let tests: any[] = $state([]);
  let helpRequests: any[] = $state([]);
  let allUsers: any[] = $state([]);
  let loading = $state(false);

  // Experiment proposal form (new_matter type)
  let expOpen = $state(false);
  let expTitle = $state('');
  let expDesc = $state('');
  let expStart = $state('');
  let expEnd = $state('');

  // Test proposal form
  let testOpen = $state(false);
  let testTitle = $state('');
  let testDesc = $state('');
  let testMethod = $state('');

  // Experiment review (observer)
  let reviewOpen = $state(false);
  let reviewTarget: any = $state(null);
  let reviewType = $state<'experiment' | 'test'>('experiment');
  let reviewStatus = $state('');
  let reviewNotes = $state('');

  // Add chemistry log form
  let logOpen = $state(false);
  let logExp: any = $state(null);
  let logDate = $state('');
  let logMatter = $state('');
  let logPersonnel = $state('');
  let logLinkedTestId = $state('');
  let logNotes = $state('');

  // View logs
  let viewLogsOpen = $state(false);
  let viewLogsExp: any = $state(null);
  let viewLogsData: any[] = $state([]);
  let viewLogsLoading = $state(false);

  // Conclusion request form (chemist)
  let conclusionReqOpen = $state(false);
  let conclusionReqExp: any = $state(null);
  let conclusionFinalNotes = $state('');
  let conclusionFinalFindings = $state('');
  let conclusionMethodology = $state('');
  let conclusionKeyResults = $state('');
  let conclusionRecommendations = $state('');
  let conclusionLimitations = $state('');

  // Conclusion review + matter creation form (observer)
  let conclusionReviewOpen = $state(false);
  let conclusionReviewTarget: any = $state(null);
  let conclusionReviewLogs: any[] = $state([]);
  let conclusionReviewLoading = $state(false);
  let conclusionDecision = $state('');
  let conclusionReviewNotes = $state('');
  let matterName = $state('');
  let matterClassification = $state('');
  let matterType = $state('');
  let matterProperties = $state('');

  // Help request form
  let helpReqOpen = $state(false);
  let hrTitle = $state('');
  let hrDesc = $state('');
  let hrCategory = $state('');
  let helpResolveOpen = $state(false);
  let helpResolveTarget: any = $state(null);
  let helpResolveStatus = $state('in_review');
  let helpResolveResponse = $state('');
  let helpRejectOpen = $state(false);
  let helpRejectTarget: any = $state(null);
  let helpRejectReason = $state('');
  let helpApproveOpen = $state(false);
  let helpApproveTarget: any = $state(null);
  let helpApproveAssigneeId = $state('');
  let helpDeliverOpen = $state(false);
  let helpDeliverTarget: any = $state(null);
  let helpDeliverResponse = $state('');

  // Observer dashboard
  let chemDashboard: any = $state(null);
  let chemDashboardLoading = $state(false);

  const isObserver = $derived(canPerform($session, 'the_observer'));
  const isChemist = $derived($session?.role_name === 'chemist' || $session?.role_name === 'physicist');
  const canReview = $derived(isObserver);
  const approvedTests = $derived(tests.filter((t: any) => t.status === 'approved'));
  const pendingConclusions = $derived(experiments.filter((e: any) => e.status === 'conclusion_requested'));

  const helpResolveOpts = [
    { value: 'in_review', label: 'Mark In Review' },
    { value: 'closed', label: 'Close' },
  ];

  const matterTypOpts = [
    { value: '', label: '— Select —' },
    { value: 'mineral', label: 'Mineral' },
    { value: 'alloy', label: 'Alloy' },
    { value: 'compound', label: 'Chemical Compound' },
    { value: 'composite', label: 'Composite Material' },
    { value: 'crystal', label: 'Crystal' },
    { value: 'polymer', label: 'Polymer' },
    { value: 'other', label: 'Other' },
  ];

  function expStatusClass(status: string | null) {
    switch (status) {
      case 'pending': return 'badge-open';
      case 'approved': return 'badge-approved';
      case 'in_progress': return 'badge-progress';
      case 'completed': return 'badge-done';
      case 'rejected': return 'badge-rejected';
      case 'conclusion_requested': return 'badge-conclude';
      default: return '';
    }
  }

  function getUserName(userId: string | undefined | null): string {
    if (!userId) return '—';
    const u = allUsers.find((u: any) => u.id === userId);
    return u ? (u.full_name ?? u.username ?? userId) : userId;
  }

  function getLinkedTestName(testId: string | null): string {
    if (!testId) return '—';
    try {
      const ids: string[] = JSON.parse(testId);
      if (Array.isArray(ids)) {
        return ids.map(id => {
          const t = tests.find((t: any) => t.id === id);
          return t ? t.title : id.slice(0, 8) + '…';
        }).join(', ');
      }
    } catch {}
    const t = tests.find((t: any) => t.id === testId);
    return t ? t.title : testId.slice(0, 8) + '…';
  }

  onMount(async () => {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      [matters, experiments, tests, helpRequests, allUsers] = await Promise.all([
        chemistryApi.getMatterArchive(s.token),
        researchApi.getExperiments(s.token),
        researchApi.getTestArchive(s.token),
        aerospaceApi.getHelpRequests(s.token),
        userApi.getAll(s.token),
      ]);
      // Filter to only new_matter type experiments
      experiments = experiments.filter((e: any) => e.experiment_type === 'new_matter');
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  // ── Experiment Proposal ──────────────────────────────────────────────────

  async function proposeExperiment() {
    const s = $session; if (!s) return;
    if (!expTitle || !expDesc) { showToast('Title and description required', 'error'); return; }
    try {
      await researchApi.proposeExperiment(
        s.token, expTitle, expDesc, 'new_matter',
        expStart ? expStart + 'T00:00:00Z' : undefined,
        expEnd ? expEnd + 'T00:00:00Z' : undefined,
      );
      showToast('Experiment proposed', 'success');
      expOpen = false; expTitle = ''; expDesc = ''; expStart = ''; expEnd = '';
      const all = await researchApi.getExperiments(s.token);
      experiments = all.filter((e: any) => e.experiment_type === 'new_matter');
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ── Test Proposal ────────────────────────────────────────────────────────

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

  // ── Review (Observer) ────────────────────────────────────────────────────

  function openReview(item: any, type: 'experiment' | 'test') {
    reviewTarget = item; reviewType = type; reviewStatus = ''; reviewNotes = ''; reviewOpen = true;
  }

  async function submitReview() {
    const s = $session; if (!s || !reviewTarget) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    if (reviewType === 'test' && reviewStatus !== 'approved' && reviewStatus !== 'rejected') {
      showToast('Test proposals can only be approved or rejected', 'error');
      return;
    }
    try {
      if (reviewType === 'experiment') {
        await researchApi.reviewExperiment(s.token, reviewTarget.id, reviewStatus, reviewNotes || undefined);
        const all = await researchApi.getExperiments(s.token);
        experiments = all.filter((e: any) => e.experiment_type === 'new_matter');
      } else {
        await researchApi.reviewTest(s.token, reviewTarget.id, reviewStatus, reviewNotes || undefined);
        tests = await researchApi.getTestArchive(s.token);
      }
      showToast('Review submitted', 'success'); reviewOpen = false;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ── Chemistry Log ────────────────────────────────────────────────────────

  function openAddLog(exp: any) {
    logExp = exp;
    logDate = new Date().toISOString().slice(0, 16);
    logMatter = ''; logPersonnel = ''; logLinkedTestId = ''; logNotes = '';
    logOpen = true;
  }

  async function submitLog() {
    const s = $session; if (!s || !logExp) return;
    if (!logDate) { showToast('Log date is required', 'error'); return; }
    if (!logLinkedTestId) { showToast('A linked test from the Test Archive is required', 'error'); return; }
    try {
      const dateVal = logDate.includes('T')
        ? new Date(logDate).toISOString()
        : new Date(logDate + 'T00:00:00Z').toISOString();
      await chemistryApi.addChemistryLog(
        s.token, logExp.id, dateVal, logLinkedTestId,
        logMatter || undefined, logPersonnel || undefined, logNotes || undefined,
      );
      showToast('Log entry added', 'success');
      logOpen = false;
      const all = await researchApi.getExperiments(s.token);
      experiments = all.filter((e: any) => e.experiment_type === 'new_matter');
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ── View Logs ────────────────────────────────────────────────────────────

  async function openViewLogs(exp: any) {
    const s = $session; if (!s) return;
    viewLogsExp = exp; viewLogsData = []; viewLogsLoading = true; viewLogsOpen = true;
    try {
      viewLogsData = await researchApi.getExperimentLogs(s.token, exp.id);
    } catch { viewLogsData = []; }
    viewLogsLoading = false;
  }

  // ── Conclusion Request (Chemist) ─────────────────────────────────────────

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
        s.token, conclusionReqExp.id, conclusionFinalNotes,
        conclusionFinalFindings || undefined, conclusionMethodology || undefined,
        conclusionKeyResults || undefined, conclusionRecommendations || undefined,
        conclusionLimitations || undefined,
      );
      showToast('Conclusion request submitted', 'success');
      conclusionReqOpen = false;
      const all = await researchApi.getExperiments(s.token);
      experiments = all.filter((e: any) => e.experiment_type === 'new_matter');
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ── Conclusion Review + Matter Archive (Observer) ─────────────────────────

  async function openConclusionReview(exp: any) {
    const s = $session; if (!s) return;
    conclusionReviewTarget = exp;
    conclusionDecision = ''; conclusionReviewNotes = '';
    matterName = exp.title ?? ''; matterClassification = '';
    matterType = ''; matterProperties = '';
    conclusionReviewLoading = true; conclusionReviewOpen = true;
    try {
      conclusionReviewLogs = await researchApi.getExperimentLogs(s.token, exp.id);
    } catch { conclusionReviewLogs = []; }
    conclusionReviewLoading = false;
  }

  async function submitConclusionReview() {
    const s = $session; if (!s || !conclusionReviewTarget) return;
    if (!conclusionDecision) { showToast('Decision required', 'error'); return; }
    if (conclusionDecision === 'approve' && !matterName.trim()) {
      showToast('Matter name is required when approving', 'error'); return;
    }
    try {
      await chemistryApi.approveChemistryConclusion(
        s.token, conclusionReviewTarget.id, conclusionDecision,
        matterName, matterClassification || undefined,
        matterType || undefined, matterProperties || undefined,
        conclusionReviewNotes || undefined,
      );
      showToast(conclusionDecision === 'approve' ? 'Conclusion approved — matter added to archive' : 'Conclusion rejected', 'success');
      conclusionReviewOpen = false;
      const [all, newMatters] = await Promise.all([
        researchApi.getExperiments(s.token),
        chemistryApi.getMatterArchive(s.token),
      ]);
      experiments = all.filter((e: any) => e.experiment_type === 'new_matter');
      matters = newMatters;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ── Help Requests ────────────────────────────────────────────────────────

  async function submitHelpRequest() {
    const s = $session; if (!s) return;
    if (!hrTitle) { showToast('Title required', 'error'); return; }
    try {
      await aerospaceApi.submitHelpRequest(s.token, hrTitle, hrDesc || undefined, hrCategory || undefined);
      showToast('Help request submitted — routed to The Observer', 'success');
      helpReqOpen = false; hrTitle = ''; hrDesc = ''; hrCategory = '';
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpResolve(req: any) {
    helpResolveTarget = req; helpResolveStatus = 'in_review'; helpResolveResponse = ''; helpResolveOpen = true;
  }

  async function submitHelpResolve() {
    const s = $session; if (!s || !helpResolveTarget) return;
    try {
      await aerospaceApi.resolveHelpRequest(s.token, helpResolveTarget.id, helpResolveStatus, helpResolveResponse || undefined);
      showToast('Help request updated', 'success'); helpResolveOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpReject(req: any) {
    helpRejectTarget = req; helpRejectReason = ''; helpRejectOpen = true;
  }

  async function submitHelpReject() {
    const s = $session; if (!s || !helpRejectTarget) return;
    if (!helpRejectReason.trim()) { showToast('Rejection reason required', 'error'); return; }
    try {
      await aerospaceApi.rejectHelpRequest(s.token, helpRejectTarget.id, helpRejectReason);
      showToast('Help request rejected', 'success'); helpRejectOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpApprove(req: any) {
    helpApproveTarget = req; helpApproveAssigneeId = ''; helpApproveOpen = true;
  }

  async function submitHelpApprove() {
    const s = $session; if (!s || !helpApproveTarget) return;
    if (!helpApproveAssigneeId) { showToast('Assignee required', 'error'); return; }
    try {
      await aerospaceApi.approveHelpRequest(s.token, helpApproveTarget.id, helpApproveAssigneeId);
      showToast('Help request approved — task created', 'success'); helpApproveOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openHelpDeliver(req: any) {
    helpDeliverTarget = req; helpDeliverResponse = ''; helpDeliverOpen = true;
  }

  async function submitHelpDeliver() {
    const s = $session; if (!s || !helpDeliverTarget) return;
    if (!helpDeliverResponse.trim()) { showToast('Response required', 'error'); return; }
    try {
      await aerospaceApi.proxyDeliverTaskResponse(s.token, helpDeliverTarget.id, helpDeliverResponse);
      showToast('Response delivered', 'success'); helpDeliverOpen = false;
      helpRequests = await aerospaceApi.getHelpRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ── Observer Dashboard ───────────────────────────────────────────────────

  async function loadChemDashboard() {
    const s = $session; if (!s) return;
    chemDashboardLoading = true;
    try {
      chemDashboard = await chemistryApi.getChemistryObserverDashboard(s.token);
    } catch (e: any) { showToast('Failed to load dashboard: ' + e, 'error'); }
    chemDashboardLoading = false;
  }

  $effect(() => {
    if (activeTab === 'observer_dashboard' && isObserver && !chemDashboard) {
      loadChemDashboard();
    }
  });
</script>

<svelte:head><title>RUSA IMS — Chemistry</title></svelte:head>

<PageShell title="Chemistry" subtitle="Matter archive, new matter experiments, test archive, and help requests">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='matter_archive'} onclick={() => activeTab='matter_archive'}>Matter Archive</button>
    <button class="tab" class:active={activeTab==='experiments'} onclick={() => activeTab='experiments'}>
      Experiments
      {#if pendingConclusions.length > 0 && isObserver}<span class="tab-badge">{pendingConclusions.length}</span>{/if}
    </button>
    <button class="tab" class:active={activeTab==='tests'} onclick={() => activeTab='tests'}>Test Archive</button>
    <button class="tab" class:active={activeTab==='help_requests'} onclick={() => activeTab='help_requests'}>
      Help Requests
      {#if helpRequests.filter((r: any) => r.status === 'open').length > 0}
        <span class="tab-badge">{helpRequests.filter((r: any) => r.status === 'open').length}</span>
      {/if}
    </button>
    {#if isObserver}
      <button class="tab" class:active={activeTab==='observer_dashboard'} onclick={() => activeTab='observer_dashboard'}>
        Observer Dashboard
      </button>
    {/if}
  </div>

  {#if loading}
    <p class="loading">Loading…</p>

  {:else if activeTab === 'matter_archive'}
    <div class="section-bar">
      <h2 class="section-title">Matter Archive</h2>
      <p class="access-note" style="margin:0">New matter is added only upon Observer-approved experiment conclusions.</p>
    </div>
    {#if matters.length === 0}
      <p class="empty">No matter in the archive yet. Discover new matter through the experiment pipeline.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Classification</th>
              <th>Properties</th>
              <th>Discovery Experiment</th>
              <th>Approved By</th>
              <th>Approved At</th>
            </tr>
          </thead>
          <tbody>
            {#each matters as m}
              <tr>
                <td><strong>{m.name}</strong></td>
                <td>{#if m.matter_type}<span class="matter-badge">{m.matter_type}</span>{:else}—{/if}</td>
                <td>{m.classification ?? '—'}</td>
                <td class="notes-preview">{m.properties ?? '—'}</td>
                <td>{m.discovery_experiment_id ? m.discovery_experiment_id.slice(0,8) + '…' : '—'}</td>
                <td>{getUserName(m.approved_by)}</td>
                <td>{m.approved_at ? new Date(m.approved_at).toLocaleDateString() : '—'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  {:else if activeTab === 'experiments'}
    <div class="section-bar">
      <h2 class="section-title">New Matter Experiments</h2>
      {#if isChemist}
        <button class="btn-primary" onclick={() => expOpen = true}>+ Propose Experiment</button>
      {/if}
    </div>
    <p class="access-note">
      ⚗ Proposing a new matter discovery is treated as an experiment. Each daily log entry <strong>must</strong> reference an approved test from the Test Archive.
      Once concluded, The Observer reviews and officially adds the matter to the archive.
    </p>
    {#if experiments.length === 0}
      <p class="empty">No chemistry experiments yet.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Start Date</th>
              <th>Proposed By</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each experiments as exp}
              <tr>
                <td>{exp.title}</td>
                <td>
                  <span class="badge {expStatusClass(exp.status)}">{exp.status ?? '—'}</span>
                  {#if exp.conclusion_approved}
                    <span class="badge badge-done" style="margin-left:4px">Concluded</span>
                  {/if}
                </td>
                <td>{exp.start_date ? new Date(exp.start_date).toLocaleDateString() : '—'}</td>
                <td>{getUserName(exp.proposed_by)}</td>
                <td class="actions-cell">
                  <button class="btn-small" onclick={() => openViewLogs(exp)}>View Logs</button>
                  {#if exp.status === 'in_progress' && (isChemist || isObserver)}
                    <button class="btn-small btn-add" onclick={() => openAddLog(exp)}>+ Log</button>
                  {/if}
                  {#if canReview && (exp.status === 'pending' || exp.status === 'approved')}
                    <button class="btn-small" onclick={() => openReview(exp, 'experiment')}>Review</button>
                  {/if}
                  {#if exp.status === 'in_progress' && exp.proposed_by === $session?.user_id}
                    <button class="btn-small btn-conclude" onclick={() => openConclusionRequest(exp)}>Request Conclusion</button>
                  {/if}
                  {#if isObserver && exp.status === 'conclusion_requested'}
                    <button class="btn-small btn-conclude" onclick={() => openConclusionReview(exp)}>Review Conclusion</button>
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
    <p class="access-note">
      📋 Chemists may propose new standardised tests to The Observer if a required procedure is not in the archive. Proposals can be approved or rejected.
    </p>
    {#if tests.length === 0}
      <p class="empty">No tests in the archive yet.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Methodology</th>
              <th>Proposed By</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each tests as t}
              <tr>
                <td>{t.title}</td>
                <td><span class="badge {t.status === 'approved' ? 'badge-approved' : t.status === 'rejected' ? 'badge-rejected' : 'badge-open'}">{t.status ?? 'pending'}</span></td>
                <td class="notes-preview">{t.methodology ?? '—'}</td>
                <td>{getUserName(t.proposed_by)}</td>
                <td class="actions-cell">
                  {#if canReview && t.status === 'pending'}
                    <button class="btn-small" onclick={() => openReview(t, 'test')}>Review</button>
                  {/if}
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
      {#if isChemist}
        <button class="btn-primary" onclick={() => helpReqOpen = true}>+ Send Help Request</button>
      {/if}
    </div>
    {#if helpRequests.length === 0}
      <p class="empty">No help requests.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Proxy Director</th>
              <th>Category</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each helpRequests as req}
              <tr>
                <td>{req.title}</td>
                <td><span class="badge {req.status === 'open' ? 'badge-open' : req.status === 'resolved' ? 'badge-done' : req.status === 'rejected' ? 'badge-rejected' : 'badge-progress'}">{req.status}</span></td>
                <td><span class="proxy-badge">{req.assigned_proxy_director}</span></td>
                <td>{req.category ?? '—'}</td>
                <td class="actions-cell">
                  {#if isObserver}
                    {#if req.status === 'open' || req.status === 'in_review'}
                      <button class="btn-small" onclick={() => openHelpResolve(req)}>Mark Review</button>
                      <button class="btn-small btn-add" onclick={() => openHelpApprove(req)}>Approve</button>
                      <button class="btn-small btn-danger-sm" onclick={() => openHelpReject(req)}>Reject</button>
                    {/if}
                    {#if req.status === 'converted'}
                      <button class="btn-small btn-observer" onclick={() => openHelpDeliver(req)}>Deliver Response</button>
                    {/if}
                  {:else}
                    {#if req.rejection_reason}
                      <span class="notes-preview" title={req.rejection_reason}>❌ {req.rejection_reason}</span>
                    {:else if req.response}
                      <span class="notes-preview" title={req.response}>✅ {req.response}</span>
                    {/if}
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  {:else if activeTab === 'observer_dashboard' && isObserver}
    <div class="section-bar">
      <h2 class="section-title">Observer Dashboard — Chemistry</h2>
      <button class="btn-secondary" onclick={loadChemDashboard}>↻ Refresh</button>
    </div>
    {#if chemDashboardLoading}
      <p class="loading">Loading dashboard…</p>
    {:else if chemDashboard}
      <div class="dashboard-section">
        <h3 class="subsection-title">Active New Matter Experiments</h3>
        {#if chemDashboard.active_experiments?.length === 0}
          <p class="empty">No active experiments.</p>
        {:else}
          <div class="table-wrap">
            <table class="data-table">
              <thead><tr><th>Title</th><th>Status</th><th>Start Date</th><th>Proposed By</th><th>Actions</th></tr></thead>
              <tbody>
                {#each chemDashboard.active_experiments as exp}
                  <tr>
                    <td>{exp.title}</td>
                    <td><span class="badge {expStatusClass(exp.status)}">{exp.status ?? '—'}</span></td>
                    <td>{exp.start_date ? new Date(exp.start_date).toLocaleDateString() : '—'}</td>
                    <td>{getUserName(exp.proposed_by)}</td>
                    <td class="actions-cell">
                      {#if exp.status === 'conclusion_requested'}
                        <button class="btn-small btn-conclude" onclick={() => openConclusionReview(exp)}>Review Conclusion</button>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
      <div class="dashboard-section">
        <h3 class="subsection-title">Pending Conclusions</h3>
        {#if chemDashboard.pending_conclusions?.length === 0}
          <p class="empty">No pending conclusions.</p>
        {:else}
          <div class="table-wrap">
            <table class="data-table">
              <thead><tr><th>Title</th><th>Requested At</th><th>Actions</th></tr></thead>
              <tbody>
                {#each chemDashboard.pending_conclusions as exp}
                  <tr>
                    <td>{exp.title}</td>
                    <td>{exp.conclusion_requested_at ? new Date(exp.conclusion_requested_at).toLocaleDateString() : '—'}</td>
                    <td class="actions-cell">
                      <button class="btn-small btn-conclude" onclick={() => openConclusionReview(exp)}>Review Conclusion</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {:else}
      <p class="empty">No dashboard data loaded.</p>
    {/if}
  {/if}
</PageShell>

<!-- Propose Experiment Modal -->
<Modal bind:open={expOpen} title="Propose New Matter Experiment">
  <div class="form">
    <p class="access-note">🔬 This proposal begins the pipeline for discovering a new matter. Once approved and conducted, you will submit daily logs (each must reference an approved test) and a final conclusion document.</p>
    <Field label="Title" bind:value={expTitle} required />
    <Field label="Description / Hypothesis" type="textarea" bind:value={expDesc} rows={4} required />
    <Field label="Start Date (optional)" type="date" bind:value={expStart} />
    <Field label="End Date (optional)" type="date" bind:value={expEnd} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => expOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={proposeExperiment}>Submit Proposal</button>
    </div>
  </div>
</Modal>

<!-- Propose Test Modal -->
<Modal bind:open={testOpen} title="Propose New Test">
  <div class="form">
    <p class="access-note">📋 Propose a standardised test to The Observer. Once approved it can be referenced in daily experiment logs.</p>
    <Field label="Title" bind:value={testTitle} required />
    <Field label="Description" type="textarea" bind:value={testDesc} rows={3} />
    <Field label="Methodology / Procedure" type="textarea" bind:value={testMethod} rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => testOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={proposeTest}>Submit Proposal</button>
    </div>
  </div>
</Modal>

<!-- Review Modal (experiment or test) -->
<Modal bind:open={reviewOpen} title="Review — {reviewType === 'experiment' ? 'Experiment' : 'Test Proposal'}">
  <div class="form">
    {#if reviewTarget}
      <div class="info-block">
        <p class="info-text"><strong>{reviewTarget.title}</strong></p>
        {#if reviewTarget.description}<p class="info-text">{reviewTarget.description}</p>{/if}
        {#if reviewTarget.methodology}<p class="info-text">Methodology: {reviewTarget.methodology}</p>{/if}
      </div>
    {/if}
    <Field label="Decision" type="select" bind:value={reviewStatus} options={[
      { value: '', label: '— Select —' },
      { value: 'approved', label: 'Approve' },
      ...(reviewType === 'experiment' ? [{ value: 'in_progress', label: 'Approve & Start' }] : []),
      { value: 'rejected', label: 'Reject' },
    ]} required />
    <Field label="Notes (optional)" type="textarea" bind:value={reviewNotes} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<!-- Add Chemistry Log Modal -->
<Modal bind:open={logOpen} title="Add Daily Log — {logExp?.title ?? ''}">
  <div class="form">
    <p class="access-note">⚠️ Each log entry must reference an approved test from the Test Archive (procedure and expected outcome).</p>
    <Field label="Log Date" type="datetime-local" bind:value={logDate} required />
    <div class="field">
      <label class="field-label">Linked Test (required) *</label>
      <select class="field-input" bind:value={logLinkedTestId}>
        <option value="">— Select an approved test —</option>
        {#each approvedTests as t}
          <option value={t.id}>{t.title}{#if t.methodology} — {t.methodology.slice(0, 60)}{/if}</option>
        {/each}
      </select>
      {#if approvedTests.length === 0}
        <p class="access-note" style="margin-top:0.25rem">No approved tests available. Propose a test first and wait for Observer approval.</p>
      {/if}
    </div>
    <Field label="Matter Being Tested / Observed" bind:value={logMatter} placeholder="e.g. Unknown mineral sample #4" />
    <Field label="Personnel Present (optional)" bind:value={logPersonnel} />
    <Field label="Notes / Findings" type="textarea" bind:value={logNotes} rows={4} />
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
      <div class="logs-scroll">
        {#each viewLogsData as log}
          <div class="log-entry">
            <div class="log-header">
              <span class="log-date">{log.log_date ?? '—'}</span>
              {#if log.personnel_present}<span class="log-personnel">👥 {log.personnel_present}</span>{/if}
            </div>
            {#if log.linked_test_ids}
              <p class="log-field"><strong>Test Referenced:</strong> <span class="test-linked">{getLinkedTestName(log.linked_test_ids)}</span></p>
            {/if}
            <!-- species_matter_tested is the shared DB column; for chemistry logs it holds the matter name -->
            {#if log.species_matter_tested}<p class="log-field"><strong>Matter:</strong> {log.species_matter_tested}</p>{/if}
            {#if log.notes}<p class="log-field">{log.notes}</p>{/if}
          </div>
        {/each}
      </div>
    {/if}
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => viewLogsOpen = false}>Close</button>
    </div>
  </div>
</Modal>

<!-- Conclusion Request Modal (Chemist) -->
<Modal bind:open={conclusionReqOpen} title="Request Conclusion — {conclusionReqExp?.title ?? ''}">
  <div class="form">
    <p class="access-note">📄 Submit your final conclusion document describing the matter's properties. The Observer will review and officially add it to the Matter Archive upon approval.</p>
    <Field label="Final Summary *" type="textarea" bind:value={conclusionFinalNotes} rows={3} required />
    <Field label="Key Findings on Matter Properties *" type="textarea" bind:value={conclusionFinalFindings} rows={3} />
    <Field label="Methodology Summary *" type="textarea" bind:value={conclusionMethodology} rows={3} required />
    <Field label="Key Results *" type="textarea" bind:value={conclusionKeyResults} rows={3} required />
    <Field label="Recommendations (optional)" type="textarea" bind:value={conclusionRecommendations} rows={2} />
    <Field label="Limitations (optional)" type="textarea" bind:value={conclusionLimitations} rows={2} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => conclusionReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionRequest}>Submit Conclusion Request</button>
    </div>
  </div>
</Modal>

<!-- Conclusion Review Modal (Observer) -->
<Modal bind:open={conclusionReviewOpen} title="Review Conclusion — {conclusionReviewTarget?.title ?? ''}">
  <div class="form">
    {#if conclusionReviewLoading}
      <p class="info-text">Loading logs…</p>
    {:else}
      {#if conclusionReviewTarget}
        <div class="info-block">
          {#if conclusionReviewTarget.final_notes}
            <p class="info-text"><strong>Final Summary:</strong> {conclusionReviewTarget.final_notes}</p>
          {/if}
          {#if conclusionReviewTarget.final_findings}
            <p class="info-text"><strong>Matter Properties:</strong> {conclusionReviewTarget.final_findings}</p>
          {/if}
          {#if conclusionReviewTarget.methodology_summary}
            <p class="info-text"><strong>Methodology:</strong> {conclusionReviewTarget.methodology_summary}</p>
          {/if}
          {#if conclusionReviewTarget.key_results}
            <p class="info-text"><strong>Key Results:</strong> {conclusionReviewTarget.key_results}</p>
          {/if}
        </div>
        <div class="subsection-title" style="margin:0.5rem 0 0.25rem">Experiment Logs ({conclusionReviewLogs.length})</div>
        <div class="logs-scroll" style="max-height:150px">
          {#each conclusionReviewLogs as log}
            <div class="log-entry">
              <div class="log-header">
                <span class="log-date">{log.log_date ?? '—'}</span>
              </div>
              {#if log.linked_test_ids}
                <p class="log-field"><strong>Test:</strong> <span class="test-linked">{getLinkedTestName(log.linked_test_ids)}</span></p>
              {/if}
              <!-- species_matter_tested is the shared DB column; for chemistry logs it holds the matter name -->
              {#if log.species_matter_tested}<p class="log-field"><strong>Matter:</strong> {log.species_matter_tested}</p>{/if}
              {#if log.notes}<p class="log-field">{log.notes}</p>{/if}
            </div>
          {/each}
        </div>
      {/if}
      <Field label="Decision" type="select" bind:value={conclusionDecision} options={[
        { value: '', label: '— Select —' },
        { value: 'approve', label: 'Approve — Add to Matter Archive' },
        { value: 'reject', label: 'Reject — Return to In Progress' },
      ]} required />
      {#if conclusionDecision === 'approve'}
        <div class="discovery-form">
          <p class="discovery-label">Matter Archive Entry</p>
          <Field label="Matter Name *" bind:value={matterName} required />
          <Field label="Classification (optional)" bind:value={matterClassification} placeholder="e.g. Silicate, Ferrous Alloy" />
          <Field label="Matter Type" type="select" bind:value={matterType} options={matterTypOpts} />
          <Field label="Properties / Description" type="textarea" bind:value={matterProperties} rows={3} placeholder="Chemical/physical properties for the archive record" />
        </div>
      {/if}
      <Field label="Review Notes (optional)" type="textarea" bind:value={conclusionReviewNotes} rows={2} />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => conclusionReviewOpen = false}>Cancel</button>
        <button class="btn-primary" onclick={submitConclusionReview} disabled={!conclusionDecision || (conclusionDecision === 'approve' && !matterName.trim())}>
          {conclusionDecision === 'approve' ? 'Approve & Add to Archive' : conclusionDecision === 'reject' ? 'Reject Conclusion' : 'Submit Review'}
        </button>
      </div>
    {/if}
  </div>
</Modal>

<!-- Help Request Modal -->
<Modal bind:open={helpReqOpen} title="Send Help Request">
  <div class="form">
    <p class="access-note">🔀 This request will be automatically routed to <strong>The Observer</strong> as your proxy director.</p>
    <Field label="Title" bind:value={hrTitle} required />
    <Field label="Description" type="textarea" bind:value={hrDesc} rows={4} />
    <Field label="Category (optional)" bind:value={hrCategory} placeholder="e.g. equipment, data, guidance" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpRequest}>Send Help Request</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpResolveOpen} title="Mark Help Request">
  <div class="form">
    {#if helpResolveTarget}
      <p class="info-text">Request: <strong>{helpResolveTarget.title}</strong></p>
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
    {/if}
    <p class="access-note">⚠️ The rejection reason will be sent to the requester. This cannot be undone.</p>
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
    {/if}
    <p class="access-note" style="background:rgba(0,200,83,0.06);border-color:rgba(0,200,83,0.2);color:#00c853">✅ Approving will create an assigned research task for the selected subordinate.</p>
    <div class="field">
      <label class="field-label">Assign Task To</label>
      <select class="field-input" bind:value={helpApproveAssigneeId}>
        <option value="">— Select user —</option>
        {#each allUsers as u}
          <option value={u.id}>{u.full_name ?? u.username}</option>
        {/each}
      </select>
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => helpApproveOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitHelpApprove} disabled={!helpApproveAssigneeId}>Approve & Create Task</button>
    </div>
  </div>
</Modal>

<Modal bind:open={helpDeliverOpen} title="Deliver Task Response">
  <div class="form">
    {#if helpDeliverTarget}
      <p class="info-text">Request: <strong>{helpDeliverTarget.title}</strong></p>
    {/if}
    <p class="access-note">📤 Deliver the completed task result back to the original requester.</p>
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
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .subsection-title { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; margin: 1.5rem 0 0.75rem; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; margin-bottom: 0.25rem; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .table-wrap { overflow-x: auto; }
  .data-table { width: 100%; border-collapse: collapse; font-family: 'Outfit', sans-serif; font-size: 0.85rem; }
  .data-table th { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; padding: 0.5rem 0.75rem; text-align: left; text-transform: uppercase; border-bottom: 1px solid #1e2d4a; }
  .data-table td { border-bottom: 1px solid #0d1a2e; color: #c8d8f0; padding: 0.6rem 0.75rem; }
  .data-table tr:hover td { background: rgba(61,127,255,0.04); }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-approved { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-rejected { background: rgba(255,68,102,0.12); color: #ff4466; }
  .badge-conclude { background: rgba(124,58,237,0.15); color: #a78bfa; }
  .matter-badge { background: rgba(0,212,255,0.1); border-radius: 3px; color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.65rem; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .proxy-badge { background: rgba(0,212,255,0.1); border-radius: 3px; color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.65rem; padding: 0.2rem 0.5rem; }
  .tab-badge { background: #7c3aed; border-radius: 10px; color: #fff; font-size: 0.6rem; font-weight: 700; margin-left: 4px; padding: 1px 6px; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.25rem 0.625rem; }
  .btn-small:hover { border-color: #3d7fff; color: #3d7fff; }
  .btn-add { border-color: #00c853; color: #00c853; }
  .btn-add:hover { border-color: #00e676; color: #00e676; }
  .btn-observer { border-color: #f59e0b; color: #f59e0b; }
  .btn-observer:hover { border-color: #fbbf24; color: #fbbf24; }
  .btn-conclude { border-color: #7c3aed; color: #a78bfa; }
  .btn-conclude:hover { border-color: #a78bfa; color: #c4b5fd; }
  .btn-danger { background: linear-gradient(135deg, #dc2626, #ef4444); border: none; border-radius: 4px; color: #fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-danger-sm { border-color: #ff4466; color: #ff4466; }
  .btn-danger-sm:hover { background: rgba(255,68,102,0.12); }
  .actions-cell { white-space: nowrap; display: flex; gap: 0.375rem; align-items: center; flex-wrap: wrap; }
  .notes-preview { max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #8fa3cc; font-size: 0.8rem; }
  .access-note { background: rgba(255,193,7,0.07); border: 1px solid rgba(255,193,7,0.2); border-radius: 4px; color: #ffc107; font-size: 0.82rem; line-height: 1.5; margin-bottom: 1rem; padding: 0.6rem 0.75rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-input { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-size: 0.85rem; padding: 0.5rem 0.75rem; width: 100%; box-sizing: border-box; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .info-block { background: rgba(61,127,255,0.05); border: 1px solid #1e2d4a; border-radius: 4px; display: flex; flex-direction: column; gap: 0.25rem; padding: 0.75rem; }
  .logs-scroll { border: 1px solid #1e2d4a; border-radius: 4px; max-height: 220px; overflow-y: auto; }
  .log-entry { border-bottom: 1px solid #1e2d4a; padding: 0.5rem 0.75rem; }
  .log-entry:last-child { border-bottom: none; }
  .log-header { display: flex; gap: 1rem; align-items: baseline; margin-bottom: 0.2rem; }
  .log-date { color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.7rem; }
  .log-personnel { color: #8fa3cc; font-size: 0.75rem; }
  .log-field { color: #c8d8f0; font-size: 0.8rem; margin: 0.2rem 0 0; }
  .test-linked { color: #a78bfa; }
  .discovery-form { background: rgba(0,200,83,0.05); border: 1px solid rgba(0,200,83,0.2); border-radius: 4px; display: flex; flex-direction: column; gap: 0.75rem; padding: 0.875rem; }
  .discovery-label { color: #00c853; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; text-transform: uppercase; margin: 0; }
  .dashboard-section { margin-bottom: 2rem; }
</style>
