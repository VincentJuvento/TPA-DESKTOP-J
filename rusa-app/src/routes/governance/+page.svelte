<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { governanceApi, userApi, generalApi, adminApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount, onDestroy } from 'svelte';

  type Tab = 'votes' | 'general_requests' | 'meetings' | 'relocations' | 'events' | 'nomad_tasks' | 'librarian' | 'accounts';
  let activeTab = $state<Tab>('votes');

  let votes: any[] = $state([]);
  let generalRequests: any[] = $state([]);
  let meetings: any[] = $state([]);
  let relocations: any[] = $state([]);
  let events: any[] = $state([]);
  let nomadTasks: any[] = $state([]);
  let archivePermissions: any[] = $state([]);
  let allUsers: any[] = $state([]);
  let allRoles: any[] = $state([]);
  let loading = $state(false);

  const role = $derived($session?.role_name ?? '');
  const tier = $derived($session?.tier ?? 0);
  const isNomad = $derived(canPerform($session, 'the_nomad'));
  const isLibrarian = $derived(canPerform($session, 'the_librarian'));
  const isDirectorOrAbove = $derived(tier >= 3);
  const isAdministrator = $derived(role === 'the_administrator');

  // Vote form
  let voteOpen = $state(false);
  let vTitle = $state('');
  let vDesc = $state('');

  // General request form
  let genReqOpen = $state(false);
  let grTitle = $state('');
  let grDesc = $state('');

  // Review general request modal
  let reviewGenOpen = $state(false);
  let selectedGenReq: any = $state(null);
  let genReviewStatus = $state('');
  let genReviewNotes = $state('');

  // Cast vote modal
  let castOpen = $state(false);
  let selectedVote: any = $state(null);
  let castDecision = $state('');
  let castReason = $state('');

  // Override vote modal (Administrator)
  let overrideOpen = $state(false);
  let overrideVote: any = $state(null);
  let overrideOutcome = $state('passed');

  // Meeting form
  let meetingOpen = $state(false);
  let mTitle = $state('');
  let mDesc = $state('');
  let mScheduled = $state('');
  let mLocation = $state('');
  // Meeting To/CC/BCC recipient state
  let mToSelected: any[] = $state([]);
  let mCcSelected: any[] = $state([]);
  let mBccSelected: any[] = $state([]);
  let mToSearch = $state('');
  let mCcSearch = $state('');
  let mBccSearch = $state('');

  // Relocation form
  let relocOpen = $state(false);
  let rStaffSelected: any = $state(null);
  let rLocation = $state('');
  let rType = $state('');
  let rStart = $state('');
  let rEnd = $state('');
  let rReason = $state('');

  // Event form
  let eventOpen = $state(false);
  let eTitle = $state('');
  let eDesc = $state('');
  let eDate = $state('');
  let eVenue = $state('');

  // Nomad task form
  let nomadTaskOpen = $state(false);
  let ntTitle = $state('');
  let ntDesc = $state('');
  let ntDue = $state('');
  let ntSelected: any = $state(null);

  // Librarian form
  let librarianOpen = $state(false);
  let libAction = $state<'restrict' | 'redact' | 'delete'>('restrict');
  let libTable = $state('');
  let libRecordId = $state('');
  let libAccessLevel = $state('restricted');
  let libReason = $state('');

  // Director Create Account form
  let createAccOpen = $state(false);
  let caUsername = $state('');
  let caEmail = $state('');
  let caPassword = $state('');
  let caFullName = $state('');
  let caRole = $state('');
  let caLocation = $state('');

  // Predefined locations for relocation dropdown
  const locationOptions = [
    { value: 'Earth Base', label: 'Earth Base' },
    { value: 'Space Station Alpha', label: 'Space Station Alpha' },
    { value: 'Space Station Beta', label: 'Space Station Beta' },
    { value: 'Mars Colony', label: 'Mars Colony' },
    { value: 'Settlement Zone A', label: 'Settlement Zone A' },
    { value: 'Settlement Zone B', label: 'Settlement Zone B' },
    { value: 'Exoplanet Camp 1', label: 'Exoplanet Camp 1' },
    { value: 'Exoplanet Camp 2', label: 'Exoplanet Camp 2' },
    { value: 'Research Outpost', label: 'Research Outpost' },
    { value: 'Deep Space', label: 'Deep Space' },
  ];

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      const base = await Promise.all([
        governanceApi.getVotes(s.token),
        governanceApi.getMeetings(s.token),
        governanceApi.getRelocations(s.token),
        governanceApi.getEvents(s.token),
        generalApi.getGeneralRequests(s.token),
      ]);
      [votes, meetings, relocations, events, generalRequests] = base;
      if (canPerform(s, 'the_nomad')) {
        nomadTasks = await governanceApi.getNomadTasks(s.token);
        allUsers = await userApi.getAll(s.token);
      }
      if (canPerform(s, 'the_librarian')) {
        try { archivePermissions = await governanceApi.getArchivePermissions(s.token); } catch {}
      }
      if (s.tier >= 3) {
        allRoles = await userApi.getRoles(s.token);
        if (!canPerform(s, 'the_nomad')) allUsers = await userApi.getAll(s.token);
      }
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function initiateVote() {
    const s = $session; if (!s) return;
    if (!vTitle) { showToast('Title required', 'error'); return; }
    try {
      await governanceApi.initiateVote(s.token, vTitle, vDesc || undefined);
      showToast('Vote initiated', 'success');
      voteOpen = false; vTitle = ''; vDesc = '';
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function castVote() {
    const s = $session; if (!s || !selectedVote) return;
    if (!castDecision) { showToast('Decision required', 'error'); return; }
    if (!castReason.trim()) { showToast('Reason is required when casting a vote', 'error'); return; }
    try {
      await governanceApi.castVote(s.token, selectedVote.id, castDecision, castReason);
      showToast('Vote cast', 'success'); castOpen = false; castReason = '';
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function interruptVote(vote: any) {
    const s = $session; if (!s) return;
    if (!confirm(`Interrupt vote "${vote.title}"? This will stop the vote immediately.`)) return;
    try {
      await governanceApi.interruptVote(s.token, vote.id);
      showToast('Vote interrupted', 'success');
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitOverrideVote() {
    const s = $session; if (!s || !overrideVote) return;
    try {
      await adminApi.overrideVote(s.token, overrideVote.id, overrideOutcome);
      showToast(`Vote overridden → ${overrideOutcome}`, 'success');
      overrideOpen = false;
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitGeneralRequest() {
    const s = $session; if (!s) return;
    if (!grTitle || !grDesc) { showToast('Title and description required', 'error'); return; }
    try {
      await generalApi.submitGeneralRequest(s.token, grTitle, grDesc);
      showToast('General request submitted — directors\' vote auto-initiated', 'success');
      genReqOpen = false; grTitle = ''; grDesc = '';
      generalRequests = await generalApi.getGeneralRequests(s.token);
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitGenReview() {
    const s = $session; if (!s || !selectedGenReq) return;
    if (!genReviewStatus) { showToast('Decision required', 'error'); return; }
    try {
      await generalApi.reviewGeneralRequest(s.token, selectedGenReq.id, genReviewStatus, genReviewNotes || undefined);
      showToast('Review submitted', 'success'); reviewGenOpen = false;
      generalRequests = await generalApi.getGeneralRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createMeeting() {
    const s = $session; if (!s) return;
    if (!mTitle) { showToast('Subject required', 'error'); return; }
    if (mToSelected.length === 0) { showToast('At least one To recipient is required', 'error'); return; }
    try {
      const toIds = mToSelected.map((u: any) => u.id);
      const ccIds = mCcSelected.map((u: any) => u.id);
      const bccIds = mBccSelected.map((u: any) => u.id);
      await governanceApi.createMeeting(s.token, mTitle, mDesc || undefined, mScheduled || undefined, mLocation || undefined, [], toIds, ccIds, bccIds);
      showToast('Meeting created', 'success');
      meetingOpen = false; mTitle = ''; mDesc = ''; mScheduled = ''; mLocation = '';
      mToSelected = []; mCcSelected = []; mBccSelected = [];
      mToSearch = ''; mCcSearch = ''; mBccSearch = '';
      meetings = await governanceApi.getMeetings(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function relocate() {
    const s = $session; if (!s) return;
    if (!rStaffSelected || !rLocation || !rType) { showToast('Staff, location, and type required', 'error'); return; }
    try {
      await governanceApi.relocateStaff(s.token, rStaffSelected.id, rLocation, rType, rStart ? rStart + 'T00:00:00Z' : undefined, rEnd ? rEnd + 'T00:00:00Z' : undefined, rReason || undefined);
      showToast('Relocation submitted', 'success');
      relocOpen = false; rStaffSelected = null; rLocation = ''; rType = ''; rStart = ''; rEnd = ''; rReason = '';
      relocations = await governanceApi.getRelocations(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function logEvent() {
    const s = $session; if (!s) return;
    if (!eTitle) { showToast('Title required', 'error'); return; }
    try {
      await governanceApi.logEvent(s.token, eTitle, eDesc || undefined, eDate ? eDate + 'T00:00:00Z' : undefined, eVenue || undefined);
      showToast('Event logged', 'success');
      eventOpen = false; eTitle = ''; eDesc = ''; eDate = ''; eVenue = '';
      events = await governanceApi.getEvents(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function nomadAssignTask() {
    const s = $session; if (!s) return;
    if (!ntSelected || !ntTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await governanceApi.nomadAssignTask(s.token, ntSelected.id, ntTitle, ntDesc || undefined, ntDue || undefined);
      showToast('Task assigned', 'success');
      nomadTaskOpen = false; ntSelected = null; ntTitle = ''; ntDesc = ''; ntDue = '';
      nomadTasks = await governanceApi.getNomadTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitLibrarianAction() {
    const s = $session; if (!s) return;
    if (!libTable || !libRecordId) { showToast('Table name and record ID required', 'error'); return; }
    try {
      if (libAction === 'restrict') {
        await governanceApi.setArchivePermission(s.token, libTable, libRecordId, libAccessLevel || 'restricted');
      } else if (libAction === 'redact') {
        await governanceApi.redactRecord(s.token, libTable, libRecordId, libReason || undefined);
      } else {
        await governanceApi.deleteRecord(s.token, libTable, libRecordId, libReason || undefined);
      }
      showToast('Action completed', 'success');
      librarianOpen = false; libTable = ''; libRecordId = ''; libReason = '';
      try { archivePermissions = await governanceApi.getArchivePermissions(s.token); } catch {}
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createAccount() {
    const s = $session; if (!s) return;
    if (!caUsername || !caEmail || !caPassword || !caFullName || !caRole) {
      showToast('All required fields must be filled', 'error'); return;
    }
    try {
      await governanceApi.directorCreateAccount(s.token, caUsername, caEmail, caPassword, caFullName, caRole, caLocation || undefined);
      showToast('Account created', 'success');
      createAccOpen = false; caUsername = ''; caEmail = ''; caPassword = ''; caFullName = ''; caRole = ''; caLocation = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const voteCols = [
    { key: 'title', label: 'Motion' },
    { key: 'status', label: 'Status' },
    { key: 'vote_type', label: 'Type' },
    { key: 'created_at', label: 'Initiated' },
  ];
  const genReqCols = [
    { key: 'title', label: 'Title' },
    { key: 'status', label: 'Status' },
    { key: 'created_at', label: 'Submitted' },
  ];
  const reviewDecisionOpts = [
    { value: 'approved', label: 'Approve' },
    { value: 'rejected', label: 'Reject' },
  ];
  const meetingCols = [
    { key: 'title', label: 'Meeting' },
    { key: 'scheduled_at', label: 'Scheduled' },
    { key: 'location', label: 'Location' },
  ];
  const relocCols = [
    { key: 'staff_name', label: 'Staff' },
    { key: 'to_location', label: 'To' },
    { key: 'relocation_type', label: 'Type' },
    { key: 'start_date', label: 'Start' },
  ];
  const eventCols = [
    { key: 'title', label: 'Event' },
    { key: 'event_date', label: 'Date' },
    { key: 'venue', label: 'Venue' },
  ];
  const nomadTaskCols = [
    { key: 'title', label: 'Task' },
    { key: 'target_role', label: 'Role' },
    { key: 'status', label: 'Status' },
    { key: 'due_date', label: 'Due' },
  ];
  const archivePermCols = [
    { key: 'table_name', label: 'Table' },
    { key: 'record_id', label: 'Record ID' },
    { key: 'access_level', label: 'Access Level' },
    { key: 'created_at', label: 'Set At' },
  ];
  const decisionOpts = [
    { value: 'yes', label: 'Yes' },
    { value: 'no', label: 'No' },
    { value: 'abstain', label: 'Abstain' },
  ];

  let now = $state(Date.now());
  const ticker = setInterval(() => { now = Date.now(); }, 1000);
  onDestroy(() => clearInterval(ticker));

  function getTimeRemaining(deadline: string | null): string {
    if (!deadline) return '';
    const diff = new Date(deadline).getTime() - now;
    if (diff <= 0) return 'Expired';
    const mins = Math.floor(diff / 60000);
    const secs = Math.floor((diff % 60000) / 1000);
    return `${mins}m ${secs.toString().padStart(2, '0')}s`;
  }

  function isExpired(deadline: string | null): boolean {
    if (!deadline) return false;
    return new Date(deadline).getTime() <= now;
  }
  const relocTypeOpts = [
    { value: 'temporary', label: 'Temporary' },
    { value: 'permanent', label: 'Permanent' },
    { value: 'transfer', label: 'Transfer' },
  ];
  const accessLevelOpts = [
    { value: 'restricted', label: 'Restricted' },
    { value: 'classified', label: 'Classified' },
    { value: 'public', label: 'Public' },
  ];
  const libActionOpts = [
    { value: 'restrict', label: 'Restrict Access' },
    { value: 'redact', label: 'Redact (Censor)' },
    { value: 'delete', label: 'Delete Record' },
  ];
  let nomadTargetOptions = $derived(
    allUsers
      .filter((u: any) => u.role_name === 'settler_commander' || u.role_name === 'head_of_sanitary')
      .map((u: any) => ({ value: u.id, label: `${u.full_name} (${u.role_display_name || u.role_name})` }))
  );

  function filteredMeetingRecipients(search: string, exclude: any[]): any[] {
    const q = search.toLowerCase();
    const excludeIds = new Set(exclude.map((u: any) => u.id));
    return allUsers.filter((u: any) =>
      !excludeIds.has(u.id) &&
      (u.username?.toLowerCase().includes(q) || u.full_name?.toLowerCase().includes(q))
    ).slice(0, 8);
  }

  let mToSuggestions = $derived(mToSearch.length > 0 ? filteredMeetingRecipients(mToSearch, mToSelected) : []);
  let mCcSuggestions = $derived(mCcSearch.length > 0 ? filteredMeetingRecipients(mCcSearch, [...mToSelected, ...mCcSelected]) : []);
  let mBccSuggestions = $derived(mBccSearch.length > 0 ? filteredMeetingRecipients(mBccSearch, [...mToSelected, ...mCcSelected, ...mBccSelected]) : []);

  let roleOptions = $derived(allRoles.map((r: any) => ({ value: r.name, label: r.display_name || r.name })));
</script>

<svelte:head><title>RUSA IMS — Governance</title></svelte:head>

<PageShell title="Governance" subtitle="Votes, meetings, relocations, and event documentation">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='votes'} onclick={() => activeTab='votes'}>Votes</button>
    <button class="tab" class:active={activeTab==='general_requests'} onclick={() => activeTab='general_requests'}>General Requests</button>
    <button class="tab" class:active={activeTab==='meetings'} onclick={() => activeTab='meetings'}>Meetings</button>
    <button class="tab" class:active={activeTab==='relocations'} onclick={() => activeTab='relocations'}>Relocations</button>
    <button class="tab" class:active={activeTab==='events'} onclick={() => activeTab='events'}>Events</button>
    {#if isNomad}
      <button class="tab" class:active={activeTab==='nomad_tasks'} onclick={() => activeTab='nomad_tasks'}>Assign Tasks</button>
    {/if}
    {#if isLibrarian}
      <button class="tab" class:active={activeTab==='librarian'} onclick={() => activeTab='librarian'}>Data Governance</button>
    {/if}
    {#if isDirectorOrAbove}
      <button class="tab" class:active={activeTab==='accounts'} onclick={() => activeTab='accounts'}>Accounts</button>
    {/if}
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'votes'}
    <div class="section-bar">
      <h2 class="section-title">Votes</h2>
      <button class="btn-primary" onclick={() => voteOpen = true}>+ Initiate Vote</button>
    </div>
    {#if votes.length === 0}
      <p class="empty">No votes found</p>
    {:else}
      <div class="vote-list">
        {#each votes as vote}
          <div class="vote-card" class:expired={vote.status !== 'open' || isExpired(vote.deadline)}>
            <div class="vote-info">
              <span class="vote-title">{vote.title}</span>
              {#if vote.description}
                <span class="vote-desc">{vote.description}</span>
              {/if}
              <span class="vote-meta">Initiated: {vote.created_at ? new Date(vote.created_at).toLocaleString() : ''}</span>
            </div>
            <div class="vote-right">
              <span class="status-badge status-{vote.status}">{vote.status}</span>
              {#if vote.vote_type}
                <span class="vote-type-badge vote-type-{vote.vote_type}">{vote.vote_type === 'general' ? '🗳 General' : vote.vote_type === 'budget' ? '💰 Budget' : vote.vote_type === 'blueprint' ? '📐 Blueprint' : '⚡ Pressing'}</span>
              {/if}
              {#if vote.status === 'open' && vote.deadline && !isExpired(vote.deadline)}
                <span class="countdown">⏱ {getTimeRemaining(vote.deadline)}</span>
                {#if isDirectorOrAbove}
                  <button class="btn-primary" onclick={() => { selectedVote = vote; castDecision = ''; castReason = ''; castOpen = true; }}>Cast Vote</button>
                {/if}
                {#if isAdministrator}
                  <button class="btn-warn" onclick={() => interruptVote(vote)}>Interrupt</button>
                  <button class="btn-danger" onclick={() => { overrideVote = vote; overrideOutcome = 'passed'; overrideOpen = true; }}>Override</button>
                {/if}
              {:else if vote.status === 'open' && isExpired(vote.deadline)}
                <span class="countdown expired-label">Resolving...</span>
                {#if isAdministrator}
                  <button class="btn-warn" onclick={() => interruptVote(vote)}>Interrupt</button>
                  <button class="btn-danger" onclick={() => { overrideVote = vote; overrideOutcome = 'passed'; overrideOpen = true; }}>Override</button>
                {/if}
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else if activeTab === 'general_requests'}
    <div class="section-bar">
      <h2 class="section-title">General Requests</h2>
      <div class="bypass-info-row">
        <span class="vote-type-badge vote-type-general">🗳 Auto-voted by Directors</span>
      </div>
      <button class="btn-primary" onclick={() => genReqOpen = true}>+ Submit Request</button>
    </div>
    <p class="vote-system-note">
      General requests automatically trigger a directors' vote upon submission. Approval is only possible after the linked vote passes.
      <strong>Data requests, conclusion requests, and security broadcasts bypass this voting system</strong> and are handled by their respective specialists.
    </p>
    {#if generalRequests.length === 0}
      <p class="empty">No general requests found</p>
    {:else}
      <div class="vote-list">
        {#each generalRequests as req}
          <div class="vote-card" class:expired={req.status !== 'under_vote' && req.status !== 'pending'}>
            <div class="vote-info">
              <span class="vote-title">{req.title}</span>
              {#if req.description}
                <span class="vote-desc">{req.description}</span>
              {/if}
              <span class="vote-meta">Submitted: {req.created_at ? new Date(req.created_at).toLocaleString() : ''}</span>
              {#if req.vote_id}
                <span class="vote-meta">Linked vote: <code class="vote-id-code">{req.vote_id}</code></span>
              {/if}
            </div>
            <div class="vote-right">
              <span class="status-badge status-{req.status}">{req.status}</span>
              {#if req.vote_id}
                <span class="vote-type-badge vote-type-general">🗳 Under Director Vote</span>
              {/if}
              {#if isDirectorOrAbove && (req.status === 'under_vote' || req.status === 'pending')}
                <button class="btn-primary" onclick={() => { selectedGenReq = req; genReviewStatus = ''; genReviewNotes = ''; reviewGenOpen = true; }}>Review</button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else if activeTab === 'meetings'}
    <div class="section-bar">
      <h2 class="section-title">Meetings</h2>
      <button class="btn-primary" onclick={() => meetingOpen = true}>+ Create Meeting</button>
    </div>
    <Table columns={meetingCols} rows={meetings} />
  {:else if activeTab === 'relocations'}
    <div class="section-bar">
      <h2 class="section-title">Staff Relocations</h2>
      <button class="btn-primary" onclick={() => relocOpen = true}>+ Relocate Staff</button>
    </div>
    <Table columns={relocCols} rows={relocations} />
  {:else if activeTab === 'events'}
    <div class="section-bar">
      <h2 class="section-title">Event Documents</h2>
      <button class="btn-primary" onclick={() => eventOpen = true}>+ Log Event</button>
    </div>
    <Table columns={eventCols} rows={events} />
  {:else if activeTab === 'nomad_tasks'}
    <div class="section-bar">
      <h2 class="section-title">Task Assignments</h2>
      <button class="btn-primary" onclick={() => nomadTaskOpen = true}>+ Assign Task</button>
    </div>
    <Table columns={nomadTaskCols} rows={nomadTasks} />
  {:else if activeTab === 'librarian'}
    <div class="section-bar">
      <h2 class="section-title">Data Governance Actions</h2>
      <button class="btn-primary" onclick={() => librarianOpen = true}>+ New Action</button>
    </div>
    <Table columns={archivePermCols} rows={archivePermissions} />
  {:else if activeTab === 'accounts'}
    <div class="section-bar">
      <h2 class="section-title">Create Staff Account</h2>
    </div>
    <div class="director-card">
      <Field label="Username" bind:value={caUsername} required />
      <div style="margin-top: 1rem;"><Field label="Email" type="email" bind:value={caEmail} required /></div>
      <div style="margin-top: 1rem;"><Field label="Password" type="password" bind:value={caPassword} required /></div>
      <div style="margin-top: 1rem;"><Field label="Full Name" bind:value={caFullName} required /></div>
      <div style="margin-top: 1rem;"><Field label="Role" type="select" bind:value={caRole} options={roleOptions} required /></div>
      <div style="margin-top: 1rem;"><Field label="Location" bind:value={caLocation} /></div>
      <div class="form-actions" style="margin-top: 1.5rem;">
        <button class="btn-primary" onclick={createAccount}>Create Account</button>
      </div>
    </div>
  {/if}
</PageShell>

<Modal bind:open={voteOpen} title="Initiate Vote">
  <div class="form">
    <Field label="Motion Title" bind:value={vTitle} required />
    <Field label="Description" type="textarea" bind:value={vDesc} rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => voteOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={initiateVote}>Initiate</button>
    </div>
  </div>
</Modal>

<Modal bind:open={castOpen} title="Cast Vote: {selectedVote?.title}">
  <div class="form">
    {#if selectedVote?.deadline}
      <p class="vote-timer">Time remaining: <span class="countdown">{getTimeRemaining(selectedVote.deadline)}</span></p>
    {/if}
    <p class="quorum-note">⚖️ Quorum: 8 of 13 Directors required. Vote resolves automatically when quorum is reached.</p>
    <Field label="Your Decision" type="select" bind:value={castDecision} options={decisionOpts} required />
    <Field label="Reason (required)" type="textarea" bind:value={castReason} rows={3} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => castOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={castVote}>Cast Vote</button>
    </div>
  </div>
</Modal>

<Modal bind:open={overrideOpen} title="Override Vote: {overrideVote?.title}">
  <div class="form">
    <div class="warning-box">⚠ Administrator override will immediately force the vote to a specific outcome, bypassing the quorum requirement.</div>
    <Field label="Forced Outcome" type="select" bind:value={overrideOutcome} options={[{value:'passed',label:'Passed'},{value:'failed',label:'Failed'}]} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => overrideOpen = false}>Cancel</button>
      <button class="btn-danger" onclick={submitOverrideVote}>Force Override</button>
    </div>
  </div>
</Modal>

<Modal bind:open={meetingOpen} title="Create Meeting">
  <div class="form">
    <div class="field">
      <label class="field-label">From</label>
      <div class="field-input from-field">{$session?.full_name || $session?.username || '—'}</div>
    </div>
    <div class="field">
      <label class="field-label">To <span class="required-star">*</span></label>
      <div class="assignee-autocomplete">
        <div class="token-list">
          {#each mToSelected as u}
            <span class="token">{u.full_name || u.username}<button class="token-remove" onclick={() => { mToSelected = mToSelected.filter((x: any) => x.id !== u.id); }}>×</button></span>
          {/each}
          <div class="autocomplete-wrap">
            <input class="field-input" bind:value={mToSearch} placeholder="Search by name or username…" />
            {#if mToSuggestions.length > 0}
              <div class="suggestions">
                {#each mToSuggestions as u}
                  <button class="suggestion-item" onclick={() => { mToSelected = [...mToSelected, u]; mToSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
    <div class="field">
      <label class="field-label">CC</label>
      <div class="assignee-autocomplete">
        <div class="token-list">
          {#each mCcSelected as u}
            <span class="token">{u.full_name || u.username}<button class="token-remove" onclick={() => { mCcSelected = mCcSelected.filter((x: any) => x.id !== u.id); }}>×</button></span>
          {/each}
          <div class="autocomplete-wrap">
            <input class="field-input" bind:value={mCcSearch} placeholder="Search by name or username…" />
            {#if mCcSuggestions.length > 0}
              <div class="suggestions">
                {#each mCcSuggestions as u}
                  <button class="suggestion-item" onclick={() => { mCcSelected = [...mCcSelected, u]; mCcSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
    <div class="field">
      <label class="field-label">BCC</label>
      <div class="assignee-autocomplete">
        <div class="token-list">
          {#each mBccSelected as u}
            <span class="token">{u.full_name || u.username}<button class="token-remove" onclick={() => { mBccSelected = mBccSelected.filter((x: any) => x.id !== u.id); }}>×</button></span>
          {/each}
          <div class="autocomplete-wrap">
            <input class="field-input" bind:value={mBccSearch} placeholder="Search by name or username…" />
            {#if mBccSuggestions.length > 0}
              <div class="suggestions">
                {#each mBccSuggestions as u}
                  <button class="suggestion-item" onclick={() => { mBccSelected = [...mBccSelected, u]; mBccSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
    <Field label="Subject" bind:value={mTitle} required />
    <Field label="Description" type="textarea" bind:value={mDesc} />
    <Field label="Scheduled At" type="datetime-local" bind:value={mScheduled} />
    <Field label="Location" bind:value={mLocation} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { meetingOpen = false; mToSelected = []; mCcSelected = []; mBccSelected = []; mToSearch = ''; mCcSearch = ''; mBccSearch = ''; }}>Cancel</button>
      <button class="btn-primary" onclick={createMeeting}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={relocOpen} title="Relocate Staff">
  <div class="form">
    <div class="field">
      <label class="field-label">Staff Member</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={rStaffSelected} />
    </div>
    <Field label="Destination" type="select" bind:value={rLocation} options={locationOptions} required />
    <Field label="Type" type="select" bind:value={rType} options={relocTypeOpts} required />
    <Field label="Start Date" type="date" bind:value={rStart} />
    <Field label="End Date" type="date" bind:value={rEnd} />
    <Field label="Reason" type="textarea" bind:value={rReason} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => relocOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={relocate}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={eventOpen} title="Log Event">
  <div class="form">
    <Field label="Title" bind:value={eTitle} required />
    <Field label="Description" type="textarea" bind:value={eDesc} />
    <Field label="Event Date" type="date" bind:value={eDate} />
    <Field label="Venue" bind:value={eVenue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => eventOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={logEvent}>Log</button>
    </div>
  </div>
</Modal>

<Modal bind:open={nomadTaskOpen} title="Assign Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To (Settler Commander or Head of Sanitary)</label>
      <UserAutocompleteSingle users={allUsers.filter((u: any) => u.role_name === 'settler_commander' || u.role_name === 'head_of_sanitary')} bind:selected={ntSelected} />
    </div>
    <Field label="Task Title" bind:value={ntTitle} required />
    <Field label="Description" type="textarea" bind:value={ntDesc} />
    <Field label="Due Date" type="datetime-local" bind:value={ntDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { nomadTaskOpen = false; ntSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={nomadAssignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={librarianOpen} title="Data Governance Action">
  <div class="form">
    <Field label="Action Type" type="select" bind:value={libAction} options={libActionOpts} required />
    <Field label="Table Name" bind:value={libTable} required placeholder="e.g. research_tasks, missions" />
    <Field label="Record ID (UUID)" bind:value={libRecordId} required />
    {#if libAction === 'restrict'}
      <Field label="Access Level" type="select" bind:value={libAccessLevel} options={accessLevelOpts} required />
    {:else}
      <Field label="Reason" type="textarea" bind:value={libReason} />
    {/if}
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => librarianOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitLibrarianAction}>Apply</button>
    </div>
  </div>
</Modal>

<Modal bind:open={genReqOpen} title="Submit General Request">
  <div class="form">
    <p class="vote-system-note">This request will automatically initiate a directors' vote upon submission. Approval requires the vote to pass.</p>
    <Field label="Title" bind:value={grTitle} required />
    <Field label="Description" type="textarea" bind:value={grDesc} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => genReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitGeneralRequest}>Submit & Initiate Vote</button>
    </div>
  </div>
</Modal>

<Modal bind:open={reviewGenOpen} title="Review General Request: {selectedGenReq?.title}">
  <div class="form">
    {#if selectedGenReq}
      <p class="vote-system-note">
        {#if selectedGenReq.vote_id}
          This request has a linked directors' vote. You can only approve it once the vote has <strong>passed</strong>.
        {:else}
          No vote is linked to this request yet.
        {/if}
      </p>
      <p class="info-text">Request: <strong>{selectedGenReq.description}</strong></p>
    {/if}
    <Field label="Decision" type="select" bind:value={genReviewStatus} options={reviewDecisionOpts} required />
    <Field label="Review Notes" type="textarea" bind:value={genReviewNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reviewGenOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitGenReview}>Submit Review</button>
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
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .vote-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .vote-card { display: flex; align-items: flex-start; justify-content: space-between; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem 1.25rem; gap: 1rem; }
  .vote-card.expired { opacity: 0.7; }
  .vote-info { display: flex; flex-direction: column; gap: 0.25rem; flex: 1; }
  .vote-title { font-size: 0.95rem; color: #e8eeff; font-weight: 500; }
  .vote-desc { font-size: 0.82rem; color: #6a7f9a; }
  .vote-meta { font-size: 0.75rem; color: #4a5d82; }
  .vote-right { display: flex; flex-direction: column; align-items: flex-end; gap: 0.5rem; flex-shrink: 0; }
  .countdown { font-family: 'Space Mono', monospace; font-size: 0.8rem; color: #ffaa00; }
  .expired-label { color: #ff4466; }
  .status-badge { display: inline-block; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.06em; }
  .status-open { background: rgba(0,212,255,0.15); color: #00d4ff; }
  .status-passed { background: rgba(0,255,128,0.12); color: #00ff80; }
  .status-failed { background: rgba(255,68,102,0.12); color: #ff4466; }
  .status-interrupted { background: rgba(255,170,0,0.12); color: #ffaa00; }
  .status-under_vote { background: rgba(255,170,0,0.12); color: #ffaa00; }
  .status-pending { background: rgba(0,212,255,0.1); color: #8fa3cc; }
  .status-approved { background: rgba(0,255,128,0.12); color: #00ff80; }
  .status-rejected { background: rgba(255,68,102,0.12); color: #ff4466; }
  .empty { color: #4a5d82; padding: 1.5rem 0; font-size: 0.9rem; }
  .vote-timer { font-size: 0.82rem; color: #8fa3cc; margin: 0; }
  .director-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; max-width: 500px; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .assignee-autocomplete { position: relative; }
  .autocomplete-wrap { position: relative; }
  .token-list { display: flex; flex-wrap: wrap; gap: 0.35rem; align-items: flex-start; border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.4rem 0.5rem; background: rgba(13,21,40,0.6); }
  .token-list .autocomplete-wrap { flex: 1; min-width: 120px; }
  .token-list .autocomplete-wrap .field-input { border: none; background: transparent; padding: 2px 0; box-shadow: none; }
  .from-field { background: rgba(13,21,40,0.4); color: #8fa3cc; cursor: default; pointer-events: none; }
  .required-star { color: #ff6444; }
  .suggestions { position: absolute; top: 100%; left: 0; right: 0; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; z-index: 100; max-height: 200px; overflow-y: auto; }
  .suggestion-item { display: block; width: 100%; background: none; border: none; border-bottom: 1px solid #1e2d4a; color: #e8eeff; cursor: pointer; font-size: 0.85rem; padding: 0.5rem 0.75rem; text-align: left; }
  .suggestion-item:last-child { border-bottom: none; }
  .suggestion-item:hover { background: rgba(61,127,255,0.1); }
  .suggestion-role { color: #4a5d82; font-size: 0.75rem; }
  .token { display: inline-flex; align-items: center; gap: 0.25rem; background: rgba(61,127,255,0.15); border: 1px solid rgba(61,127,255,0.3); border-radius: 12px; color: #8fa3cc; font-size: 0.8rem; padding: 2px 8px; }
  .token-remove { background: none; border: none; color: #8fa3cc; cursor: pointer; font-size: 0.9rem; line-height: 1; padding: 0 2px; }
  .token-remove:hover { color: #ff4466; }
  .vote-type-badge { display: inline-flex; align-items: center; gap: 0.25rem; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.05em; }
  .vote-type-general { background: rgba(0,212,255,0.1); color: #00d4ff; border: 1px solid rgba(0,212,255,0.2); }
  .vote-type-budget { background: rgba(255,200,0,0.1); color: #ffc800; border: 1px solid rgba(255,200,0,0.25); }
  .vote-type-pressing_issue { background: rgba(255,100,68,0.1); color: #ff6444; border: 1px solid rgba(255,100,68,0.25); }
  .vote-type-blueprint { background: rgba(156,39,176,0.12); color: #ce93d8; border: 1px solid rgba(156,39,176,0.3); }
  .bypass-info-row { display: flex; align-items: center; gap: 0.5rem; flex: 1; padding-left: 1rem; }
  .vote-system-note { background: rgba(0,212,255,0.05); border: 1px solid rgba(0,212,255,0.15); border-radius: 4px; color: #8fa3cc; font-size: 0.82rem; margin-bottom: 0.75rem; padding: 0.6rem 0.85rem; }
  .vote-id-code { background: rgba(255,255,255,0.05); border-radius: 3px; color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.72rem; padding: 1px 4px; }
  .info-text { color: #8fa3cc; font-size: 0.88rem; margin: 0; }
  .quorum-note { background: rgba(61,127,255,0.07); border: 1px solid rgba(61,127,255,0.2); border-radius: 4px; color: #8fa3cc; font-size: 0.8rem; margin: 0; padding: 0.5rem 0.75rem; }
  .warning-box { background: rgba(255,68,102,0.1); border: 1px solid rgba(255,68,102,0.3); border-radius: 4px; color: #ff4466; font-size: 0.82rem; padding: 0.75rem; }
  .btn-warn { background: none; border: 1px solid #ffaa00; border-radius: 4px; color: #ffaa00; cursor: pointer; font-size: 0.82rem; font-weight: 600; padding: 0.5rem 0.9rem; }
  .btn-danger { background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.82rem; font-weight: 600; padding: 0.5rem 0.9rem; }
</style>
