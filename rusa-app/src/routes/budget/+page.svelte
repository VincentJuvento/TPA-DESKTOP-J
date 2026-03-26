<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { budgetApi, governanceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'requests' | 'expenditures';
  let activeTab = $state<Tab>('requests');

  let requests: any[] = $state([]);
  let expenditures: any[] = $state([]);
  let loading = $state(false);

  const role = $derived($session?.role_name ?? '');
  const tier = $derived($session?.tier ?? 0);
  const isAccountant = $derived(role === 'the_accountant');
  const canInitiateVote = $derived(isAccountant || tier >= 3);
  const canSubmitRequests = $derived(tier >= 2 && !isAccountant);

  // Review modals
  let reviewOpen = $state(false);
  let selectedItem: any = $state(null);
  let reviewType = $state<'budget' | 'expenditure'>('budget');
  let reviewStatus = $state('');
  let reviewNotes = $state('');
  let flagReason = $state('');
  let flagOpen = $state(false);

  // Budget Request Form
  let budgetReqOpen = $state(false);
  let budgetTitle = $state('');
  let budgetDesc = $state('');
  let budgetAmount = $state('');
  let budgetItems = $state('');

  // Expenditure Report Form
  let expReportOpen = $state(false);
  let expTitle = $state('');
  let expDesc = $state('');
  let expAmount = $state('');
  let expItems = $state('');
  let expInvoiceFile: FileList | null = $state(null);

  // Investigation form
  let invOpen = $state(false);
  let invTitle = $state('');
  let invDesc = $state('');
  let invRelated = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [requests, expenditures] = await Promise.all([
        budgetApi.getBudgetRequests(s.token),
        budgetApi.getExpenditures(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  function openReview(item: any, type: 'budget' | 'expenditure') {
    selectedItem = item; reviewType = type; reviewStatus = ''; reviewNotes = ''; reviewOpen = true;
  }

  function openFlag(item: any, type: 'budget' | 'expenditure') {
    selectedItem = item; reviewType = type; flagReason = ''; flagOpen = true;
  }

  async function initiateVoteForRequest(item: any) {
    const s = $session; if (!s) return;
    try {
      const voteId = await budgetApi.initiateBudgetVote(s.token, item.id);
      showToast(`Directors' vote initiated (ID: ${voteId})`, 'success');
      requests = await budgetApi.getBudgetRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitReview() {
    const s = $session; if (!s || !selectedItem) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      await budgetApi.reviewBudget(s.token, selectedItem.id, reviewStatus, reviewNotes || undefined);
      showToast('Review submitted', 'success'); reviewOpen = false;
      requests = await budgetApi.getBudgetRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFlag() {
    const s = $session; if (!s || !selectedItem) return;
    if (!flagReason) { showToast('Reason required', 'error'); return; }
    try {
      if (reviewType === 'budget') {
        await budgetApi.flagBudget(s.token, selectedItem.id, flagReason);
        requests = await budgetApi.getBudgetRequests(s.token);
      } else {
        await budgetApi.flagExpenditure(s.token, selectedItem.id, flagReason);
        expenditures = await budgetApi.getExpenditures(s.token);
      }
      showToast('Flagged for review', 'warning'); flagOpen = false;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitBudgetReq() {
    const s = $session; if (!s) return;
    if (!budgetTitle || !budgetDesc || !budgetAmount || !budgetItems) { showToast('All fields required', 'error'); return; }
    try {
      const reqId = await budgetApi.submitBudgetRequest(s.token, budgetTitle, budgetDesc, Number(budgetAmount), budgetItems);
      // Auto-route to Directors' Voting System
      await budgetApi.initiateBudgetVote(s.token, reqId);
      showToast('Budget request submitted and routed to Directors\' Vote', 'success');
      budgetReqOpen = false; budgetTitle = ''; budgetDesc = ''; budgetAmount = ''; budgetItems = '';
      requests = await budgetApi.getBudgetRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitExpReport() {
    const s = $session; if (!s) return;
    if (!expTitle || !expDesc || !expAmount || !expItems || !expInvoiceFile || expInvoiceFile.length === 0) { 
      showToast('All fields including invoice upload are required', 'error'); return; 
    }
    try {
      // Dummy processing for invoice file upload
      const invoiceData = `Uploaded: ${expInvoiceFile[0].name}`;
      await budgetApi.submitExpenditureReport(s.token, expTitle, expDesc, Number(expAmount), expItems, invoiceData);
      showToast('Expenditure report submitted to The Accountant', 'success');
      expReportOpen = false; expTitle = ''; expDesc = ''; expAmount = ''; expItems = ''; expInvoiceFile = null;
      expenditures = await budgetApi.getExpenditures(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitInvestigation() {
    const s = $session; if (!s) return;
    if (!invTitle || !invDesc) { showToast('Title and description required', 'error'); return; }
    try {
      // Mark the report as flagged/under investigation
      if (invRelated) {
        await budgetApi.flagExpenditure(s.token, invRelated, invDesc);
      }

      const invId = await budgetApi.submitInvestigation(s.token, invTitle, invDesc, invRelated || undefined);
      
      // Auto-initiate meeting with Directors and involved party
      if (invRelated) {
        // Find the involved party
        const report = expenditures.find(e => e.id === invRelated);
        if (report && report.reported_by) {
          const allUsers = await userApi.getAll(s.token);
          const directors = allUsers.filter(u => u.tier >= 3);
          const directorIds = directors.map(d => d.id);
          const attendeeIds = [...new Set([...directorIds, report.reported_by])];
          
          await governanceApi.createMeeting(
            s.token,
            `Investigation Meeting: ${invTitle}`,
            `Automated meeting for investigation ${invId}. \nDescription: ${invDesc}`,
            undefined, // scheduledAt
            'Virtual Boardroom',
            attendeeIds
          );
          showToast('Investigation submitted, report marked, and Meeting initiated', 'success');
        } else {
          showToast('Investigation submitted and report marked', 'success');
        }
      } else {
        showToast('Investigation submitted', 'success');
      }
      
      invOpen = false; invTitle = ''; invDesc = ''; invRelated = '';
      expenditures = await budgetApi.getExpenditures(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const expCols = [
    { key: 'title', label: 'Title' },
    { key: 'department', label: 'Department' },
    { key: 'total_amount', label: 'Amount' },
    { key: 'status', label: 'Status' },
  ];
  const statusOpts = [
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
    { value: 'pending', label: 'Pending' },
  ];
</script>

<svelte:head><title>RUSA IMS — Budget & Finance</title></svelte:head>

<PageShell title="Budget & Finance" subtitle="Budget requests, expenditure reports, and investigations">
  <div class="header-row">
    <div class="tabs">
      <button class="tab" class:active={activeTab==='requests'} onclick={() => activeTab='requests'}>Budget Requests</button>
      <button class="tab" class:active={activeTab==='expenditures'} onclick={() => activeTab='expenditures'}>Expenditure Reports</button>
    </div>
    <div class="actions">
      {#if canSubmitRequests}
        {#if activeTab === 'requests'}
          <button class="btn-primary" onclick={() => budgetReqOpen = true}>+ Submit Budget Request</button>
        {:else if activeTab === 'expenditures'}
          <button class="btn-primary" onclick={() => expReportOpen = true}>+ Submit Expenditure Report</button>
        {/if}
      {/if}
    </div>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'requests'}
    <p class="vote-system-note">
      💰 <strong>Budget requests require a collective directors' vote</strong> before they can be approved.
      Use "Initiate Director Vote" on any pending request to start the vote. Approval is blocked until the vote passes.
    </p>
    {#if requests.length === 0}
      <p class="empty">No budget requests found</p>
    {:else}
      <div class="request-list">
        {#each requests as req}
          <div class="request-card">
            <div class="request-info">
              <span class="request-title">{req.title}</span>
              {#if req.description}
                <span class="request-desc">{req.description}</span>
              {/if}
              <span class="request-meta">Amount: <strong>${req.amount ?? '—'}</strong></span>
              {#if req.accountant_notes}
                <span class="request-meta">Notes: {req.accountant_notes}</span>
              {/if}
            </div>
            <div class="request-right">
              <span class="status-badge status-{req.status}">{req.status}</span>
              {#if req.vote_id}
                <span class="vote-badge vote-badge-linked">🗳 Directors' Vote Required</span>
              {:else if req.status === 'pending'}
                <span class="vote-badge vote-badge-missing">⚠ No Vote Yet</span>
              {/if}
              <div class="request-actions">
                {#if canInitiateVote && req.status === 'pending' && !req.vote_id}
                  <button class="btn-vote" onclick={() => initiateVoteForRequest(req)}>Initiate Director Vote</button>
                {/if}
                {#if isAccountant && (req.status === 'pending' || req.status === 'under_review')}
                  <button class="btn-primary" onclick={() => openReview(req, 'budget')}>Review</button>
                  <button class="btn-warning-sm" onclick={() => openFlag(req, 'budget')}>Flag</button>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else if activeTab === 'expenditures'}
    <Table columns={expCols} rows={expenditures} onRowClick={(r) => openReview(r, 'expenditure')} />
  {/if}
</PageShell>

<Modal bind:open={reviewOpen} title="Review: {selectedItem?.title}">
  <div class="form">
    <p class="info-text">Amount: <strong>{selectedItem?.amount || selectedItem?.total_amount}</strong></p>
    {#if selectedItem?.vote_id}
      <p class="vote-requirement-note">
        🗳 This budget request has a linked directors' vote. <strong>Approval requires the vote to have passed.</strong>
        If the vote has not passed, the approval will be blocked.
      </p>
    {:else}
      <p class="vote-warning-note">
        ⚠ <strong>No directors' vote is linked to this request.</strong>
        Per RUSA policy, budget requests require a collective directors' vote before approval.
        Close this review and use "Initiate Director Vote" first.
      </p>
    {/if}
    <Field label="Decision" type="select" bind:value={reviewStatus} options={statusOpts} required />
    <Field label="Notes" type="textarea" bind:value={reviewNotes} />
    <div class="form-actions">
      {#if isAccountant && reviewType === 'expenditure'}
        <button class="btn-warning" onclick={() => { reviewOpen = false; invRelated = selectedItem.id; invOpen = true; }}>Submit Investigation</button>
      {/if}
      <button class="btn-warning" onclick={() => { reviewOpen = false; openFlag(selectedItem, reviewType); }}>Flag</button>
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<Modal bind:open={flagOpen} title="Flag for Investigation">
  <div class="form">
    <Field label="Reason" type="textarea" bind:value={flagReason} required rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => flagOpen = false}>Cancel</button>
      <button class="btn-warning" onclick={submitFlag}>Flag</button>
    </div>
  </div>
</Modal>

<Modal bind:open={invOpen} title="Submit Investigation">
  <div class="form">
    <Field label="Title" bind:value={invTitle} required />
    <Field label="Description" type="textarea" bind:value={invDesc} required rows={4} />
    <Field label="Related Report ID (optional)" bind:value={invRelated} disabled />
    <p class="info-text">Submitting this will mark the report and automatically initiate a Meeting invite to The Directors and involved parties.</p>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitInvestigation}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={budgetReqOpen} title="Submit Budget Request">
  <div class="form">
    <Field label="Request Title" bind:value={budgetTitle} required />
    <Field label="Description" type="textarea" bind:value={budgetDesc} required rows={3} />
    <Field label="Total Amount" type="number" bind:value={budgetAmount} required />
    <Field label="Itemized List" type="textarea" bind:value={budgetItems} required rows={5} hint="Please provide an itemized and descriptive list of requested funds." />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => budgetReqOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitBudgetReq}>Submit Request</button>
    </div>
  </div>
</Modal>

<Modal bind:open={expReportOpen} title="Submit Expenditure Report">
  <div class="form">
    <Field label="Report Title" bind:value={expTitle} required />
    <Field label="Description" type="textarea" bind:value={expDesc} required rows={3} />
    <Field label="Total Amount Spent" type="number" bind:value={expAmount} required />
    <Field label="Itemized Descriptions" type="textarea" bind:value={expItems} required rows={5} hint="Please provide itemized descriptions of how approved funds were spent." />
    <div class="field">
      <label class="field-label" for="invoice-upload">Invoice Attachment <span class="required">*</span></label>
      <input type="file" id="invoice-upload" class="field-input" bind:files={expInvoiceFile} required />
      <span class="field-hint">A mandatory file upload/attachment field for an Invoice.</span>
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => expReportOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitExpReport}>Submit Report</button>
    </div>
  </div>
</Modal>

<style>
  .header-row { display: flex; align-items: flex-end; justify-content: space-between; margin-bottom: 1.5rem; }
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1.5rem 0; font-size: 0.9rem; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .vote-system-note { background: rgba(255,200,0,0.06); border: 1px solid rgba(255,200,0,0.2); border-radius: 4px; color: #8fa3cc; font-size: 0.82rem; margin-bottom: 1rem; padding: 0.6rem 0.85rem; }
  .vote-requirement-note { background: rgba(0,212,255,0.05); border: 1px solid rgba(0,212,255,0.2); border-radius: 4px; color: #8fa3cc; font-size: 0.82rem; padding: 0.6rem 0.85rem; }
  .vote-warning-note { background: rgba(255,170,0,0.07); border: 1px solid rgba(255,170,0,0.25); border-radius: 4px; color: #ffaa00; font-size: 0.82rem; padding: 0.6rem 0.85rem; }
  .request-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .request-card { display: flex; align-items: flex-start; justify-content: space-between; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem 1.25rem; gap: 1rem; }
  .request-info { display: flex; flex-direction: column; gap: 0.25rem; flex: 1; }
  .request-title { color: #e8eeff; font-size: 0.95rem; font-weight: 500; }
  .request-desc { color: #6a7f9a; font-size: 0.82rem; }
  .request-meta { color: #4a5d82; font-size: 0.75rem; }
  .request-right { display: flex; flex-direction: column; align-items: flex-end; gap: 0.5rem; flex-shrink: 0; }
  .request-actions { display: flex; flex-wrap: wrap; gap: 0.4rem; justify-content: flex-end; }
  .status-badge { display: inline-block; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.06em; }
  .status-pending { background: rgba(0,212,255,0.1); color: #8fa3cc; }
  .status-approved { background: rgba(0,255,128,0.12); color: #00ff80; }
  .status-rejected { background: rgba(255,68,102,0.12); color: #ff4466; }
  .status-flagged { background: rgba(255,170,0,0.12); color: #ffaa00; }
  .status-under_review { background: rgba(255,200,0,0.1); color: #ffc800; }
  .vote-badge { display: inline-block; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.05em; }
  .vote-badge-linked { background: rgba(255,200,0,0.1); color: #ffc800; border: 1px solid rgba(255,200,0,0.25); }
  .vote-badge-missing { background: rgba(255,100,68,0.1); color: #ff6444; border: 1px solid rgba(255,100,68,0.2); }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.5rem 1rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-warning { background: none; border: 1px solid #ffaa00; border-radius: 4px; color: #ffaa00; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-warning-sm { background: none; border: 1px solid #ffaa00; border-radius: 4px; color: #ffaa00; cursor: pointer; font-size: 0.75rem; padding: 0.4rem 0.75rem; }
  .btn-vote { background: rgba(255,200,0,0.1); border: 1px solid rgba(255,200,0,0.4); border-radius: 4px; color: #ffc800; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.05em; padding: 0.4rem 0.75rem; text-transform: uppercase; }
  .btn-vote:hover { background: rgba(255,200,0,0.15); }
  
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .required { color: #ff4466; margin-left: 2px; }
  .field-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.625rem 0.875rem; transition: border-color 0.15s; width: 100%; box-sizing: border-box; }
  .field-input:focus { outline: none; border-color: #3d7fff; box-shadow: 0 0 0 2px rgba(61,127,255,0.15); }
  .field-hint { font-size: 0.75rem; color: #4a5d82; }
</style>
