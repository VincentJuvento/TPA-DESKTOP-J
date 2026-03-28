<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Field from '$lib/components/Field.svelte';
  import WorkspaceDetail from '$lib/components/WorkspaceDetail.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { researchApi, chemistryApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  // Note: experiment logs for both research and chemistry flows are loaded via
  // researchApi.getExperimentLogs because both use the shared experiment_logs table.

  const expId = $derived($page.params.id);

  let experiment: any = $state(null);
  let logs: any[] = $state([]);
  let tests: any[] = $state([]);
  let allUsers: any[] = $state([]);
  let loading = $state(true);
  let logsLoading = $state(false);

  type DetailTab = 'overview' | 'activity_logs';
  let activeTab = $state<DetailTab>('overview');

  const isChemistryExp = $derived(experiment?.experiment_type === 'new_matter');
  const backPath = $derived(isChemistryExp ? '/chemistry' : '/research');

  const canReviewExperiment = $derived(
    isChemistryExp
      ? canPerform($session, 'the_observer')
      : ($session?.role_name === 'the_observer' ||
         $session?.role_name === 'the_taskmaster' ||
         ($session?.tier ?? 0) >= 3)
  );
  const isObserver = $derived(canPerform($session, 'the_observer'));
  const isTaskmaster = $derived(canPerform($session, 'the_taskmaster'));
  const approvedTests = $derived(tests.filter((t: any) => t.status === 'approved'));
  const isAuthorizedForConclusion = $derived(
    experiment ? experiment.reviewed_by === $session?.user_id : false
  );
  const canAddLog = $derived(
    experiment?.status === 'in_progress' || experiment?.status === 'approved'
  );
  const canRequestConclusion = $derived(
    experiment?.status === 'in_progress' && experiment?.proposed_by === $session?.user_id
  );
  const canReviewConclusion = $derived(
    experiment?.status === 'conclusion_requested' &&
    (isChemistryExp ? isObserver : isTaskmaster)
  );
  const workspaceTags = $derived([
    ...(experiment?.experiment_type ? [{ label: `🔬 ${experiment.experiment_type}`, className: 'tag-type' }] : []),
    { label: experiment?.status ?? '—', className: `badge ${expStatusBadgeClass(experiment?.status)}` },
    ...(experiment?.conclusion_approved ? [{ label: 'Concluded', className: 'badge badge-done' }] : []),
    ...(experiment?.start_date ? [{ label: `📅 ${new Date(experiment.start_date).toLocaleDateString()}`, className: 'tag-date' }] : []),
  ]);
  const workspaceActions = $derived([
    ...(canReviewExperiment && (experiment?.status === 'pending' || experiment?.status === 'approved')
      ? [{ label: 'Review', className: 'btn-secondary', onClick: () => { reviewStatus = ''; reviewNotes = ''; reviewOpen = true; } }]
      : []),
    ...(isObserver && !isChemistryExp
      ? [{ label: 'Assign Task', className: 'btn-observer', onClick: () => { assignTaskAssignee = null; assignTaskTitle = ''; assignTaskDue = ''; assignTaskOpen = true; } }]
      : []),
    ...(canRequestConclusion
      ? [{ label: 'Request Conclusion', className: 'btn-conclude', onClick: () => openConclusionRequest() }]
      : []),
    ...(canReviewConclusion
      ? [{ label: 'Review Conclusion', className: 'btn-conclude', onClick: () => openConclusionReview() }]
      : []),
  ]);
  const workspaceTabs = $derived([
    { key: 'overview', label: 'Overview' },
    { key: 'activity_logs', label: 'Activity Logs', count: logs.length },
  ]);

  // Review modal
  let reviewOpen = $state(false);
  let reviewStatus = $state('');
  let reviewNotes = $state('');
  const reviewStatusOpts = $derived(isChemistryExp
    ? [
        { value: '', label: '— Select —' },
        { value: 'approved', label: 'Approve' },
        { value: 'in_progress', label: 'Approve & Start' },
        { value: 'rejected', label: 'Reject' },
      ]
    : [
        { value: '', label: '— Select —' },
        { value: 'approved', label: 'Approved' },
        { value: 'rejected', label: 'Rejected' },
        { value: 'in_progress', label: 'Approve & Start' },
      ]
  );

  // Assign Task modal (Observer, non-chemistry only)
  let assignTaskOpen = $state(false);
  let assignTaskAssignee: any = $state(null);
  let assignTaskTitle = $state('');
  let assignTaskDue = $state('');

  // Add Log modal (shared fields)
  let logOpen = $state(false);
  let logDate = $state('');
  let logPersonnel = $state('');
  let logNotes = $state('');
  // Research-only log fields
  let logSpecies = $state('');
  let logTestsText = $state('');
  let logLinkedTestIds: string[] = $state([]);
  let logNewSpecies = $state(false);
  let logNewSpeciesName = $state('');
  let logNewSpeciesClass = $state('');
  let logNewSpeciesHabitat = $state('');
  let logNewSpeciesDesc = $state('');
  // Chemistry-only log fields
  let chemLogMatter = $state('');
  let chemLogLinkedTestId = $state('');

  // Conclusion Request modal
  let conclusionReqOpen = $state(false);
  let conclusionFinalNotes = $state('');
  let conclusionFinalFindings = $state('');
  let conclusionMethodology = $state('');
  let conclusionKeyResults = $state('');
  let conclusionRecommendations = $state('');
  let conclusionLimitations = $state('');

  // Conclusion Review modal
  let conclusionReviewOpen = $state(false);
  let conclusionReviewLogs: any[] = $state([]);
  let conclusionDecision = $state('');
  let conclusionReviewNotes = $state('');
  let conclusionReviewLoading = $state(false);
  // Chemistry-specific conclusion fields
  let matterName = $state('');
  let matterClassification = $state('');
  let matterType = $state('');
  let matterProperties = $state('');

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

  onMount(async () => {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      const [allExps, allTests, users] = await Promise.all([
        researchApi.getExperiments(s.token),
        researchApi.getTestArchive(s.token),
        userApi.getAll(s.token),
      ]);
      experiment = allExps.find((e: any) => e.id === expId) ?? null;
      tests = allTests;
      allUsers = users;
      if (!experiment) {
        showToast('Experiment not found', 'error');
        goto('/research');
        return;
      }
      logsLoading = true;
      logs = await researchApi.getExperimentLogs(s.token, expId);
      logsLoading = false;
    } catch (e: any) {
      showToast('Failed to load: ' + e, 'error');
    }
    loading = false;
  });

  function getUserName(userId: string | undefined | null): string {
    if (!userId) return '—';
    const u = allUsers.find((u: any) => u.id === userId);
    return u ? (u.full_name ?? u.username ?? userId) : userId;
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

  async function refreshExperiment() {
    const s = $session; if (!s) return;
    const allExps = await researchApi.getExperiments(s.token);
    experiment = allExps.find((e: any) => e.id === expId) ?? null;
  }

  async function submitReview() {
    const s = $session; if (!s || !experiment) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      await researchApi.reviewExperiment(s.token, experiment.id, reviewStatus, reviewNotes || undefined);
      showToast('Review submitted', 'success');
      reviewOpen = false; reviewStatus = ''; reviewNotes = '';
      await refreshExperiment();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitAssignTask() {
    const s = $session; if (!s || !experiment) return;
    if (!assignTaskAssignee || !assignTaskTitle.trim()) { showToast('Assignee and title required', 'error'); return; }
    try {
      await researchApi.assignExperimentTask(
        s.token, experiment.id, assignTaskAssignee.id, assignTaskTitle,
        assignTaskDue ? new Date(assignTaskDue).toISOString() : undefined,
      );
      showToast('Task assigned', 'success');
      assignTaskOpen = false; assignTaskAssignee = null; assignTaskTitle = ''; assignTaskDue = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openAddLog() {
    logDate = new Date().toISOString().slice(0, 16);
    logPersonnel = ''; logNotes = '';
    logSpecies = ''; logTestsText = ''; logLinkedTestIds = [];
    logNewSpecies = false; logNewSpeciesName = ''; logNewSpeciesClass = '';
    logNewSpeciesHabitat = ''; logNewSpeciesDesc = '';
    chemLogMatter = ''; chemLogLinkedTestId = '';
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
    const s = $session; if (!s || !experiment) return;
    if (!logDate) { showToast('Log date is required', 'error'); return; }
    try {
      const dateVal = logDate.includes('T')
        ? new Date(logDate).toISOString()
        : new Date(logDate + 'T00:00:00Z').toISOString();
      if (isChemistryExp) {
        if (!chemLogLinkedTestId) { showToast('A linked test is required for chemistry logs', 'error'); return; }
        await chemistryApi.addChemistryLog(
          s.token, experiment.id, dateVal, chemLogLinkedTestId,
          chemLogMatter || undefined, logPersonnel || undefined, logNotes || undefined,
        );
      } else {
        const linkedJson = logLinkedTestIds.length > 0 ? JSON.stringify(logLinkedTestIds) : undefined;
        await researchApi.addLog(
          s.token, experiment.id, dateVal,
          logPersonnel || undefined, logSpecies || undefined,
          logTestsText || undefined, linkedJson, logNotes || undefined,
        );
        if (logNewSpecies && logNewSpeciesName.trim()) {
          try {
            await researchApi.proposeSpeciesFromDiscovery(
              s.token, experiment.id, logNewSpeciesName,
              logNewSpeciesDesc || undefined, logNewSpeciesClass || undefined, logNewSpeciesHabitat || undefined,
            );
            showToast('New species discovery proposed for archive', 'success');
          } catch (se: any) {
            showToast('Log saved but species proposal failed: ' + se, 'error');
          }
        }
      }
      showToast('Log entry added', 'success');
      logOpen = false;
      logs = await researchApi.getExperimentLogs(s.token, experiment.id);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function getLinkedTestNames(linkedTestIds: string | null): string {
    if (!linkedTestIds) return '';
    try {
      const ids: string[] = JSON.parse(linkedTestIds);
      if (Array.isArray(ids)) {
        return ids.map(id => {
          const t = tests.find((t: any) => t.id === id);
          return t ? t.title : id.slice(0, 8) + '…';
        }).join(', ');
      }
    } catch {}
    const t = tests.find((t: any) => t.id === linkedTestIds);
    return t ? t.title : linkedTestIds.slice(0, 8) + '…';
  }

  function openConclusionRequest() {
    conclusionFinalNotes = experiment?.final_notes ?? '';
    conclusionFinalFindings = experiment?.final_findings ?? '';
    conclusionMethodology = experiment?.methodology_summary ?? '';
    conclusionKeyResults = experiment?.key_results ?? '';
    conclusionRecommendations = experiment?.recommendations ?? '';
    conclusionLimitations = experiment?.limitations ?? '';
    conclusionReqOpen = true;
  }

  async function submitConclusionRequest() {
    const s = $session; if (!s || !experiment) return;
    if (!conclusionFinalNotes.trim()) { showToast('Final summary is required', 'error'); return; }
    if (!conclusionMethodology.trim()) { showToast('Methodology summary is required', 'error'); return; }
    if (!conclusionKeyResults.trim()) { showToast('Key results are required', 'error'); return; }
    try {
      await researchApi.requestConclusion(
        s.token, experiment.id, conclusionFinalNotes,
        conclusionFinalFindings || undefined, conclusionMethodology || undefined,
        conclusionKeyResults || undefined, conclusionRecommendations || undefined,
        conclusionLimitations || undefined,
      );
      showToast('Conclusion request submitted', 'success');
      conclusionReqOpen = false;
      await refreshExperiment();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openConclusionReview() {
    const s = $session; if (!s) return;
    conclusionDecision = ''; conclusionReviewNotes = '';
    matterName = experiment?.title ?? ''; matterClassification = ''; matterType = ''; matterProperties = '';
    conclusionReviewLoading = true; conclusionReviewOpen = true;
    try {
      conclusionReviewLogs = await researchApi.getExperimentLogs(s.token, experiment.id);
    } catch { conclusionReviewLogs = []; }
    conclusionReviewLoading = false;
  }

  async function submitConclusionReview() {
    const s = $session; if (!s || !experiment) return;
    if (!conclusionDecision) { showToast('Decision required', 'error'); return; }
    if (!isChemistryExp && !isAuthorizedForConclusion) {
      showToast('Only the original approver can approve this conclusion', 'error'); return;
    }
    try {
      if (isChemistryExp) {
        if (conclusionDecision === 'approve' && !matterName.trim()) {
          showToast('Matter name is required when approving', 'error'); return;
        }
        await chemistryApi.approveChemistryConclusion(
          s.token, experiment.id, conclusionDecision,
          matterName, matterClassification || undefined,
          matterType || undefined, matterProperties || undefined,
          conclusionReviewNotes || undefined,
        );
      } else {
        await researchApi.approveConclusion(
          s.token, experiment.id, conclusionDecision, conclusionReviewNotes || undefined,
        );
      }
      showToast(conclusionDecision === 'approve' ? 'Conclusion approved' : 'Conclusion rejected', 'success');
      conclusionReviewOpen = false;
      await refreshExperiment();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }
</script>

<svelte:head><title>RUSA IMS — {experiment?.title ?? 'Experiment'}</title></svelte:head>

<PageShell title={experiment?.title ?? 'Experiment'} subtitle="Experiment Workspace">
  {#if loading}
    <p class="loading">Loading experiment…</p>
  {:else if !experiment}
    <p class="empty">Experiment not found.</p>
  {:else}
    <WorkspaceDetail
      backLabel="Back"
      onBack={() => goto(backPath)}
      tags={workspaceTags}
      actions={workspaceActions}
      tabs={workspaceTabs}
      activeTab={activeTab}
      onSelectTab={(t) => activeTab = t as DetailTab}
    >
      {#snippet body()}
        {#if activeTab === 'overview'}
      <div class="overview-grid">
        {#if experiment.description}
          <div class="detail-card full-width">
            <h3 class="detail-label">Description</h3>
            <div class="readonly-block">{experiment.description}</div>
          </div>
        {/if}
        <div class="detail-row">
          <div class="detail-card">
            <h3 class="detail-label">Proposed By</h3>
            <p class="detail-value">{getUserName(experiment.proposed_by)}</p>
          </div>
          <div class="detail-card">
            <h3 class="detail-label">Reviewed By</h3>
            <p class="detail-value">{getUserName(experiment.reviewed_by)}</p>
          </div>
          {#if experiment.start_date}
            <div class="detail-card">
              <h3 class="detail-label">Start Date</h3>
              <p class="detail-value">{new Date(experiment.start_date).toLocaleDateString()}</p>
            </div>
          {/if}
          {#if experiment.end_date}
            <div class="detail-card">
              <h3 class="detail-label">End Date</h3>
              <p class="detail-value">{new Date(experiment.end_date).toLocaleDateString()}</p>
            </div>
          {/if}
        </div>
        {#if experiment.final_notes}
          <div class="detail-card full-width">
            <h3 class="detail-label">Executive Summary</h3>
            <div class="readonly-block">{experiment.final_notes}</div>
          </div>
        {/if}
        {#if experiment.methodology_summary}
          <div class="detail-card full-width">
            <h3 class="detail-label">Methodology</h3>
            <div class="readonly-block">{experiment.methodology_summary}</div>
          </div>
        {/if}
        {#if experiment.key_results}
          <div class="detail-card full-width">
            <h3 class="detail-label">Key Results</h3>
            <div class="readonly-block">{experiment.key_results}</div>
          </div>
        {/if}
        {#if experiment.final_findings}
          <div class="detail-card full-width">
            <h3 class="detail-label">Final Findings</h3>
            <div class="readonly-block">{experiment.final_findings}</div>
          </div>
        {/if}
        {#if experiment.recommendations}
          <div class="detail-card full-width">
            <h3 class="detail-label">Recommendations</h3>
            <div class="readonly-block">{experiment.recommendations}</div>
          </div>
        {/if}
        {#if experiment.limitations}
          <div class="detail-card full-width">
            <h3 class="detail-label">Limitations</h3>
            <div class="readonly-block">{experiment.limitations}</div>
          </div>
        {/if}
        {#if !experiment.description && !experiment.final_notes && !experiment.methodology_summary}
          <p class="empty">No additional details available for this experiment.</p>
        {/if}
      </div>

        {:else if activeTab === 'activity_logs'}
      <div class="section-bar">
        <h2 class="section-title">Activity Logs</h2>
        {#if canAddLog}
          <button class="btn-primary" onclick={openAddLog}>+ Log</button>
        {/if}
      </div>
      {#if logsLoading}
        <p class="loading">Loading logs…</p>
      {:else if logs.length === 0}
        <p class="empty">No log entries yet for this experiment.</p>
      {:else}
        <div class="log-list">
          {#each logs as log}
            <div class="log-card">
              <div class="log-header">
                <span class="log-date">{log.log_date ?? '—'}</span>
                {#if log.personnel_present}<span class="log-personnel">👥 {log.personnel_present}</span>{/if}
              </div>
              {#if log.linked_test_ids}
                <p class="log-field"><strong>Tests Linked:</strong> <span class="test-linked">{getLinkedTestNames(log.linked_test_ids)}</span></p>
              {:else if log.tests_performed}
                <p class="log-field"><strong>Tests:</strong> {log.tests_performed}</p>
              {/if}
              {#if log.species_matter_tested}
                <p class="log-field"><strong>Subject:</strong> {log.species_matter_tested}</p>
              {/if}
              {#if log.notes}<p class="log-field">{log.notes}</p>{/if}
              {#if log.new_species_proposed}
                <p class="log-field discovery-badge">🔬 New species proposed to archive</p>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
        {/if}
      {/snippet}
    </WorkspaceDetail>
  {/if}
</PageShell>

<!-- Review Modal -->
<Modal bind:open={reviewOpen} title="Review — {experiment?.title ?? ''}">
  <div class="form">
    <Field label="Decision" type="select" bind:value={reviewStatus} options={reviewStatusOpts} required />
    <Field label="Notes (optional)" type="textarea" bind:value={reviewNotes} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview} disabled={!reviewStatus}>Submit Review</button>
    </div>
  </div>
</Modal>

<!-- Assign Task Modal (Research experiments only) -->
<Modal bind:open={assignTaskOpen} title="Assign Task to Experiment">
  <div class="form">
    <div class="info-block">
      <p class="info-text">Experiment: <strong>{experiment?.title ?? '—'}</strong></p>
    </div>
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={assignTaskAssignee} />
    </div>
    <Field label="Task Title" bind:value={assignTaskTitle} required />
    <Field label="Due Date" type="datetime-local" bind:value={assignTaskDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => assignTaskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitAssignTask}>Assign Task</button>
    </div>
  </div>
</Modal>

<!-- Add Log Modal -->
<Modal bind:open={logOpen} title="Add Daily Log — {experiment?.title ?? ''}">
  <div class="form">
    {#if isChemistryExp}
      <p class="access-note">⚠️ Each log entry must reference an approved test from the Test Archive.</p>
    {/if}
    <Field label="Log Date" type="datetime-local" bind:value={logDate} required />
    {#if isChemistryExp}
      <div class="field">
        <label class="field-label">Linked Test (required) *</label>
        <select class="field-input" bind:value={chemLogLinkedTestId}>
          <option value="">— Select an approved test —</option>
          {#each approvedTests as t}
            <option value={t.id}>{t.title}{#if t.methodology} — {t.methodology.slice(0, 60)}{/if}</option>
          {/each}
        </select>
        {#if approvedTests.length === 0}
          <p class="access-note" style="margin-top:0.25rem">No approved tests available. Propose a test first.</p>
        {/if}
      </div>
      <Field label="Matter Being Tested / Observed" bind:value={chemLogMatter} placeholder="e.g. Unknown mineral sample #4" />
    {:else}
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
      <Field label="Species / Matter Tested" bind:value={logSpecies} placeholder="What was tested today" />
    {/if}
    <Field label="Personnel Present (optional)" bind:value={logPersonnel} />
    <Field label="Notes / Progress" type="textarea" bind:value={logNotes} rows={3} />

    {#if !isChemistryExp}
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
    {/if}

    <div class="form-actions">
      <button class="btn-secondary" onclick={() => logOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitLog}>Save Log</button>
    </div>
  </div>
</Modal>

<!-- Conclusion Request Modal -->
<Modal bind:open={conclusionReqOpen} title="Request Conclusion — {experiment?.title ?? ''}">
  <div class="form">
    {#if isChemistryExp}
      <p class="access-note">📄 Submit your final conclusion document describing the matter's properties. The Observer will review and officially add it to the Matter Archive upon approval.</p>
    {/if}
    <Field label="Executive Summary *" type="textarea" bind:value={conclusionFinalNotes} rows={4} required hint="High-level summary of the experiment outcome." />
    <Field label="Methodology Summary *" type="textarea" bind:value={conclusionMethodology} rows={3} required hint="How the experiment was conducted." />
    <Field label="Key Results / Findings *" type="textarea" bind:value={conclusionKeyResults} rows={3} required hint="Main findings and results observed." />
    <Field label="Final Findings (Detail)" type="textarea" bind:value={conclusionFinalFindings} rows={3} hint="Detailed findings and statistical results (if applicable)." />
    <Field label="Recommendations (optional)" type="textarea" bind:value={conclusionRecommendations} rows={3} />
    <Field label="Limitations (optional)" type="textarea" bind:value={conclusionLimitations} rows={3} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => conclusionReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionRequest}>Submit Conclusion Request</button>
    </div>
  </div>
</Modal>

<!-- Conclusion Review Modal -->
<Modal bind:open={conclusionReviewOpen} title="Review Conclusion — {experiment?.title ?? ''}">
  <div class="form">
    {#if conclusionReviewLoading}
      <p class="info-text">Loading…</p>
    {:else}
      {#if experiment}
        <div class="info-block">
          <p class="info-text">Type: <strong>{experiment.experiment_type ?? '—'}</strong></p>
          <p class="info-text">Requested: <strong>{experiment.conclusion_requested_at ? new Date(experiment.conclusion_requested_at).toLocaleString() : '—'}</strong></p>
          {#if !isChemistryExp}
            <p class="info-text">Original Approver: <strong>{getUserName(experiment.reviewed_by)}</strong></p>
            {#if isAuthorizedForConclusion}
              <p class="auth-badge auth-ok">✓ You are authorized to approve this conclusion</p>
            {:else}
              <p class="auth-badge auth-denied">✗ Not authorized — only the original proposal approver can approve this conclusion</p>
            {/if}
          {/if}
        </div>
        {#if experiment.final_notes}
          <div class="field">
            <span class="field-label">Executive Summary</span>
            <div class="readonly-block">{experiment.final_notes}</div>
          </div>
        {/if}
        {#if experiment.methodology_summary}
          <div class="field">
            <span class="field-label">Methodology</span>
            <div class="readonly-block">{experiment.methodology_summary}</div>
          </div>
        {/if}
        {#if experiment.key_results}
          <div class="field">
            <span class="field-label">Key Results</span>
            <div class="readonly-block">{experiment.key_results}</div>
          </div>
        {/if}
        {#if experiment.final_findings}
          <div class="field">
            <span class="field-label">Final Findings</span>
            <div class="readonly-block">{experiment.final_findings}</div>
          </div>
        {/if}
      {/if}
      <div class="field">
        <span class="field-label">Experiment Logs ({conclusionReviewLogs.length})</span>
        {#if conclusionReviewLogs.length === 0}
          <p class="info-text">No logs found.</p>
        {:else}
          <div class="logs-scroll">
            {#each conclusionReviewLogs as log}
              <div class="log-entry">
                <span class="log-date">{log.log_date ?? '—'}</span>
                {#if log.linked_test_ids}
                  <p class="log-field"><strong>Tests:</strong> {getLinkedTestNames(log.linked_test_ids)}</p>
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
      <Field label="Decision" type="select" bind:value={conclusionDecision} options={isChemistryExp
        ? [{ value: '', label: '— Select —' }, { value: 'approve', label: 'Approve — Add to Matter Archive' }, { value: 'reject', label: 'Reject — Return to In Progress' }]
        : [{ value: '', label: '— Select —' }, { value: 'approve', label: 'Approve' }, { value: 'reject', label: 'Reject' }]
      } required />
      {#if isChemistryExp && conclusionDecision === 'approve'}
        <div class="discovery-form">
          <p class="discovery-label">Matter Archive Entry</p>
          <Field label="Matter Name *" bind:value={matterName} required />
          <Field label="Classification (optional)" bind:value={matterClassification} placeholder="e.g. Silicate, Ferrous Alloy" />
          <Field label="Matter Type" type="select" bind:value={matterType} options={matterTypOpts} />
          <Field label="Properties / Description" type="textarea" bind:value={matterProperties} rows={3} />
        </div>
      {/if}
      <Field label="Review Notes (optional)" type="textarea" bind:value={conclusionReviewNotes} rows={3} />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => conclusionReviewOpen = false}>Cancel</button>
        <button class="btn-danger" onclick={submitConclusionReview}
          disabled={!conclusionDecision || (!isChemistryExp && !isAuthorizedForConclusion) || (isChemistryExp && conclusionDecision === 'approve' && !matterName.trim())}>
          {conclusionDecision === 'approve' ? 'Approve Conclusion' : conclusionDecision === 'reject' ? 'Reject Conclusion' : 'Submit Decision'}
        </button>
      </div>
    {/if}
  </div>
</Modal>

<style>
  .meta-tag {
    border-radius: 3px;
    font-family: 'Space Mono', monospace;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    padding: 0.2rem 0.5rem;
    text-transform: uppercase;
  }
  .tag-type { background: rgba(61,127,255,0.1); color: #8fa3cc; }
  .tag-date { background: rgba(0,212,255,0.08); color: #8fa3cc; }
  /* header + tabs styles moved to WorkspaceDetail component */
  .overview-grid { display: flex; flex-direction: column; gap: 1rem; }
  .detail-row { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 0.75rem; }
  .detail-card { background: rgba(13,21,40,0.6); border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.75rem 1rem; }
  .full-width { grid-column: 1 / -1; }
  .detail-label { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; margin: 0 0 0.35rem; text-transform: uppercase; }
  .detail-value { color: #c8d8f0; font-size: 0.9rem; margin: 0; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .log-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .log-card { background: rgba(13,21,40,0.6); border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.875rem 1rem; }
  .log-header { display: flex; gap: 1rem; align-items: baseline; margin-bottom: 0.35rem; }
  .log-date { color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.7rem; }
  .log-personnel { color: #8fa3cc; font-size: 0.75rem; }
  .log-field { color: #c8d8f0; font-size: 0.85rem; margin: 0.2rem 0 0; }
  .test-linked { color: #a78bfa; }
  .discovery-badge { background: rgba(0,200,83,0.08); border-radius: 3px; color: #00c853; display: inline-block; font-size: 0.8rem; padding: 2px 6px; }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-approved { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-rejected { background: rgba(255,68,102,0.12); color: #ff4466; }
  .badge-conclude { background: rgba(124,58,237,0.15); color: #a78bfa; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-observer { background: none; border: 1px solid #f59e0b; border-radius: 4px; color: #f59e0b; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-observer:hover { border-color: #fbbf24; color: #fbbf24; }
  .btn-conclude { background: none; border: 1px solid #7c3aed; border-radius: 4px; color: #a78bfa; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-conclude:hover { border-color: #a78bfa; color: #c4b5fd; }
  .btn-danger { background: linear-gradient(135deg, #dc2626, #ef4444); border: none; border-radius: 4px; color: #fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-input { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-size: 0.85rem; padding: 0.5rem 0.75rem; width: 100%; box-sizing: border-box; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; margin-bottom: 0.25rem; }
  .info-block { background: rgba(61,127,255,0.05); border: 1px solid #1e2d4a; border-radius: 4px; display: flex; flex-direction: column; gap: 0.25rem; padding: 0.75rem; }
  .readonly-block { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #c8d8f0; font-size: 0.85rem; line-height: 1.6; min-height: 60px; padding: 0.625rem 0.875rem; white-space: pre-wrap; }
  .logs-scroll { border: 1px solid #1e2d4a; border-radius: 4px; max-height: 220px; overflow-y: auto; }
  .log-entry { border-bottom: 1px solid #1e2d4a; padding: 0.5rem 0.75rem; }
  .log-entry:last-child { border-bottom: none; }
  .auth-badge { border-radius: 3px; font-size: 0.8rem; font-weight: 700; padding: 0.35rem 0.625rem; }
  .auth-ok { background: rgba(0,200,83,0.12); color: #00c853; }
  .auth-denied { background: rgba(255,68,102,0.12); color: #ff4466; }
  .access-note { background: rgba(255,193,7,0.07); border: 1px solid rgba(255,193,7,0.2); border-radius: 4px; color: #ffc107; font-size: 0.82rem; line-height: 1.5; margin-bottom: 0.5rem; padding: 0.6rem 0.75rem; }
  .test-checklist { border: 1px solid #1e2d4a; border-radius: 4px; max-height: 150px; overflow-y: auto; padding: 0.375rem; display: flex; flex-direction: column; gap: 0.25rem; }
  .test-check-item { display: flex; align-items: flex-start; gap: 0.5rem; cursor: pointer; padding: 0.25rem 0.375rem; border-radius: 3px; }
  .test-check-item:hover { background: rgba(61,127,255,0.06); }
  .test-check-label { color: #c8d8f0; font-size: 0.85rem; }
  .test-check-meta { color: #4a5d82; font-size: 0.75rem; margin-left: auto; }
  .discovery-toggle { display: flex; align-items: center; gap: 0.5rem; cursor: pointer; }
  .discovery-form { background: rgba(0,200,83,0.05); border: 1px solid rgba(0,200,83,0.2); border-radius: 4px; display: flex; flex-direction: column; gap: 0.75rem; padding: 0.875rem; }
  .discovery-label { color: #00c853; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; text-transform: uppercase; margin: 0; }
</style>
