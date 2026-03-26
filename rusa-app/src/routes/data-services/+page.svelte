<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import MarkdownOutput from '$lib/components/MarkdownOutput.svelte';
  import { session } from '$lib/stores/auth';
  import { dataApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  let requests: any[] = $state([]);
  let loading = $state(false);
  let createOpen = $state(false);
  let activeTab = $state('');

  // New request form
  let title = $state('');
  let dataType = $state('');
  let additionalContext = $state('');
  let requesterLocation = $state('');
  let requesterTelFax = $state('N/A');
  let requesterDepartment = $state('');
  let requesterDepartmentEmail = $state('');
  let requestedItems = $state<string[]>(['']);
  let reasonOfRequest = $state('');

  // Statistician review
  let reviewOpen = $state(false);
  let selectedReq: any = $state(null);
  let reviewStatus = $state('');
  let reviewNotes = $state('');
  let assignedTo = $state('');
  let allUsers: any[] = $state([]);

  // Analyst workspace
  let analyzeOpen = $state(false);
  let activeRequest: any = $state(null);
  let rawData = $state('');
  let analysisResults: string[] = $state([]);
  let filterCol = $state('');
  let filterVal = $state('');
  let pivotGroupCol = $state('');
  let pivotAggCol = $state('');

  // Submit results
  let submitOpen = $state(false);
  let submitReq: any = $state(null);
  let responseMarkdown = $state('');
  let responseStatus = $state<'provided' | 'rejected'>('provided');
  let responseExplanation = $state('');
  let analystNotes = $state('');
  let providedBy = $state<string[]>([]);
  let selectedFiles = $state<File[]>([]);

  let attachmentsOpen = $state(false);
  let attachmentsReq: any = $state(null);
  let attachments = $state<any[]>([]);

  const role = $derived($session?.role_name ?? '');

  function initTab() {
    if (role === 'the_statistician') activeTab = 'pending';
    else if (role === 'data_analyst') activeTab = 'approved';
    else activeTab = 'mine';
  }

  onMount(async () => {
    const s = $session; if (!s) return;
    initTab();
    loading = true;
    try {
      requests = await dataApi.getRequests(s.token);
      if (role === 'the_statistician') {
        allUsers = await userApi.getAll(s.token);
      } else if (role !== 'data_analyst') {
        requesterLocation = s.location ?? '';
        requesterTelFax = s.tel_fax ?? 'N/A';
        requesterDepartment = s.department ?? s.role_display_name ?? '';
        requesterDepartmentEmail = s.department_email ?? s.email ?? '';
      } else {
        allUsers = await userApi.getAll(s.token);
      }
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function reload() {
    const s = $session; if (!s) return;
    requests = await dataApi.getRequests(s.token);
  }

  async function create() {
    const s = $session; if (!s) return;
    const items = requestedItems.map(i => i.trim()).filter(Boolean);
    if (!title.trim()) { showToast('Title required', 'error'); return; }
    if (!items.length) { showToast('Requested data must be itemized', 'error'); return; }
    if (!reasonOfRequest.trim()) { showToast('Reason of request required', 'error'); return; }
    if (!requesterLocation.trim()) { showToast('Location required', 'error'); return; }
    if (!requesterTelFax.trim()) { showToast('Tel/Fax required', 'error'); return; }
    if (!requesterDepartment.trim()) { showToast('Department required', 'error'); return; }
    if (!requesterDepartmentEmail.trim()) { showToast('Department email required', 'error'); return; }
    const requestedDataItems = items.map(i => `- ${i}`).join('\n');
    try {
      await dataApi.submitRequest(s.token, {
        title,
        requestedDataItems,
        reasonOfRequest,
        description: additionalContext.trim() || undefined,
        dataType: dataType || undefined,
        requesterLocation,
        requesterTelFax,
        requesterDepartment,
        requesterDepartmentEmail
      });
      showToast('Request submitted', 'success');
      createOpen = false;
      title = '';
      dataType = '';
      additionalContext = '';
      requestedItems = [''];
      reasonOfRequest = '';
      await reload();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function addRequestedItem() {
    requestedItems = [...requestedItems, ''];
  }

  function removeRequestedItem(i: number) {
    if (requestedItems.length <= 1) return;
    requestedItems = requestedItems.filter((_, idx) => idx !== i);
  }

  function openReview(req: any) {
    selectedReq = req; reviewStatus = ''; reviewNotes = ''; assignedTo = '';
    reviewOpen = true;
  }

  async function submitReview(status: string) {
    const s = $session; if (!s || !selectedReq) return;
    try {
      await dataApi.review(s.token, selectedReq.id, status, reviewNotes || undefined, assignedTo || undefined);
      showToast('Request ' + status, 'success');
      reviewOpen = false;
      await reload();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function startProcessing(req: any) {
    const s = $session; if (!s) return;
    try {
      await dataApi.startProcessing(s.token, req.id);
      showToast('Started processing', 'success');
      await reload();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openAnalyze(req: any) {
    activeRequest = req; rawData = ''; analysisResults = [];
    filterCol = ''; filterVal = ''; pivotGroupCol = ''; pivotAggCol = '';
    analyzeOpen = true;
  }

  function openSubmit(req: any) {
    const s = $session;
    submitReq = req;
    responseMarkdown = '';
    responseStatus = 'provided';
    responseExplanation = '';
    analystNotes = '';
    providedBy = s ? [s.user_id] : [];
    selectedFiles = [];
    submitOpen = true;
  }

  function openSubmitWithData(req: any, data: string) {
    const s = $session;
    submitReq = req;
    responseMarkdown = data;
    responseStatus = 'provided';
    responseExplanation = '';
    analystNotes = '';
    providedBy = s ? [s.user_id] : [];
    selectedFiles = [];
    analyzeOpen = false;
    submitOpen = true;
  }

  function openRejectAsResponse(req: any) {
    const s = $session;
    submitReq = req;
    responseMarkdown = '';
    responseStatus = 'rejected';
    responseExplanation = '';
    analystNotes = '';
    providedBy = s ? [s.user_id] : [];
    selectedFiles = [];
    submitOpen = true;
  }

  function toggleProvidedBy(id: string) {
    if (providedBy.includes(id)) providedBy = providedBy.filter(x => x !== id);
    else providedBy = [...providedBy, id];
  }

  function onFilesSelected(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const files = input.files ? Array.from(input.files) : [];
    selectedFiles = files;
  }

  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const res = reader.result;
        if (typeof res !== 'string') return reject(new Error('Invalid file encoding'));
        const idx = res.indexOf(',');
        resolve(idx >= 0 ? res.slice(idx + 1) : res);
      };
      reader.onerror = () => reject(new Error('Failed to read file'));
      reader.readAsDataURL(file);
    });
  }

  async function doAnalystSubmit() {
    const s = $session; if (!s || !submitReq) return;
    if (!providedBy.length) { showToast('Provided By is required', 'error'); return; }
    if (responseStatus === 'provided' && !responseMarkdown.trim()) { showToast('Provided data output required', 'error'); return; }
    if (responseStatus === 'rejected' && !responseExplanation.trim()) { showToast('Explanation required for rejected responses', 'error'); return; }
    try {
      const attachments = await Promise.all(selectedFiles.map(async f => ({
        filename: f.name,
        mimeType: f.type || null,
        base64: await fileToBase64(f)
      })));

      await dataApi.analystSubmit(s.token, {
        requestId: submitReq.id,
        responseMarkdown,
        responseStatus,
        responseExplanation: responseStatus === 'rejected' ? responseExplanation : undefined,
        analystNotes: analystNotes || undefined,
        providedBy,
        attachments
      });
      showToast('Submitted to Statistician', 'success');
      submitOpen = false;
      await reload();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function deliver(req: any) {
    const s = $session; if (!s) return;
    try {
      await dataApi.deliver(s.token, req.id);
      showToast('Delivered to requester', 'success');
      await reload();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function acknowledge(req: any) {
    const s = $session; if (!s) return;
    try {
      await dataApi.acknowledge(s.token, req.id);
      showToast('Acknowledged', 'success');
      await reload();
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openAttachments(req: any) {
    const s = $session; if (!s) return;
    try {
      attachmentsReq = req;
      attachments = await dataApi.listAttachments(s.token, req.id);
      attachmentsOpen = true;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function downloadFromBase64(filename: string, mimeType: string | null, base64: string) {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const blob = new Blob([bytes], { type: mimeType || 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  }

  async function downloadAttachment(attachmentId: string) {
    const s = $session; if (!s) return;
    try {
      const file = await dataApi.downloadAttachment(s.token, attachmentId);
      downloadFromBase64(file.filename, file.mime_type ?? null, file.base64);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  // ---- Analysis tools ----
  function parseData(): string[][] {
    const lines = rawData.trim().split('\n').filter(l => l.trim());
    return lines.map(l => l.split(',').map(c => c.trim()));
  }

  function numCol(rows: string[][], col: number): number[] {
    return rows.map(r => parseFloat(r[col])).filter(v => !isNaN(v));
  }

  function runFilter() {
    const rows = parseData();
    const ci = parseInt(filterCol);
    if (isNaN(ci)) { addResult('Filter: invalid column index'); return; }
    const filtered = rows.filter(r => r[ci] === filterVal);
    addResult('Filter col=' + filterCol + ' val="' + filterVal + '":\n' + filtered.map(r => r.join(', ')).join('\n'));
  }

  function runMean() {
    const rows = parseData(); const ci = parseInt(filterCol);
    if (isNaN(ci)) { addResult('Mean: enter column index in the column field'); return; }
    const nums = numCol(rows, ci);
    if (!nums.length) { addResult('Mean: no numeric data in column ' + ci); return; }
    addResult('Mean of col ' + ci + ': ' + (nums.reduce((a, b) => a + b, 0) / nums.length).toFixed(4));
  }

  function runMedian() {
    const rows = parseData(); const ci = parseInt(filterCol);
    if (isNaN(ci)) { addResult('Median: enter column index'); return; }
    const sorted = [...numCol(rows, ci)].sort((a, b) => a - b);
    if (!sorted.length) { addResult('Median: no numeric data'); return; }
    const mid = Math.floor(sorted.length / 2);
    const median = sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
    addResult('Median of col ' + ci + ': ' + median);
  }

  function runMode() {
    const rows = parseData(); const ci = parseInt(filterCol);
    if (isNaN(ci)) { addResult('Mode: enter column index'); return; }
    const vals = rows.map(r => r[ci]).filter(Boolean);
    const freq: Record<string, number> = {};
    vals.forEach(v => { freq[v] = (freq[v] || 0) + 1; });
    const max = Math.max(...Object.values(freq));
    const modes = Object.entries(freq).filter(([, c]) => c === max).map(([v]) => v);
    addResult('Mode of col ' + ci + ': ' + modes.join(', ') + ' (freq=' + max + ')');
  }

  function runVariance() {
    const rows = parseData(); const ci = parseInt(filterCol);
    if (isNaN(ci)) { addResult('Variance: enter column index'); return; }
    const nums = numCol(rows, ci);
    if (!nums.length) { addResult('Variance: no numeric data'); return; }
    const mean = nums.reduce((a, b) => a + b, 0) / nums.length;
    const variance = nums.reduce((a, b) => a + (b - mean) ** 2, 0) / nums.length;
    addResult('Variance of col ' + ci + ': ' + variance.toFixed(4));
  }

  function runMinMax() {
    const rows = parseData(); const ci = parseInt(filterCol);
    if (isNaN(ci)) { addResult('Min/Max: enter column index'); return; }
    const nums = numCol(rows, ci);
    if (!nums.length) { addResult('Min/Max: no numeric data'); return; }
    addResult('Col ' + ci + ' — Min: ' + Math.min(...nums) + ', Max: ' + Math.max(...nums));
  }

  function runPivot() {
    const rows = parseData();
    const gi = parseInt(pivotGroupCol); const ai = parseInt(pivotAggCol);
    if (isNaN(gi) || isNaN(ai)) { addResult('Pivot: enter group and aggregate column indices'); return; }
    const groups: Record<string, number[]> = {};
    rows.forEach(r => {
      const key = r[gi] || '(empty)';
      const val = parseFloat(r[ai]);
      if (!isNaN(val)) { groups[key] = groups[key] || []; groups[key].push(val); }
    });
    const lines = Object.entries(groups).map(([k, vs]) => {
      const avg = vs.reduce((a, b) => a + b, 0) / vs.length;
      return k + ': count=' + vs.length + ', sum=' + vs.reduce((a, b) => a + b, 0).toFixed(2) + ', avg=' + avg.toFixed(2);
    });
    addResult('Pivot (group col=' + gi + ', agg col=' + ai + '):\n' + lines.join('\n'));
  }

  function addResult(text: string) {
    analysisResults = [...analysisResults, text];
  }

  function buildChart(type: 'bar' | 'line'): string {
    const rows = parseData(); const ci = parseInt(filterCol);
    if (isNaN(ci) || rows.length === 0) return '';
    // Only use pure numeric values — parseFloat already filters non-numbers
    const nums = rows.map(r => parseFloat(r[ci])).filter(v => !isNaN(v) && isFinite(v));
    if (!nums.length) return '';
    const w = 400; const h = 180; const pad = 30;
    const max = Math.max(...nums); const min = Math.min(...nums);
    const range = max - min || 1;
    const barW = Math.floor((w - pad * 2) / nums.length);
    // All interpolated values are derived from integers/floats — no raw user strings
    if (type === 'bar') {
      const bars = nums.map((v, i) => {
        const bh = Math.max(0, Math.round(((v - min) / range) * (h - pad * 2)));
        const x = Math.round(pad + i * barW); const y = Math.round(h - pad - bh);
        return `<rect x="${x}" y="${y}" width="${barW - 2}" height="${bh}" fill="#3d7fff" opacity="0.8"/>`;
      }).join('');
      return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" style="background:#0d1528">${bars}</svg>`;
    } else {
      const pts = nums.map((v, i) => {
        const x = Math.round(pad + i * barW + barW / 2);
        const y = Math.round(h - pad - Math.max(0, ((v - min) / range) * (h - pad * 2)));
        return `${x},${y}`;
      }).join(' ');
      return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" style="background:#0d1528"><polyline points="${pts}" fill="none" stroke="#00d4ff" stroke-width="2"/></svg>`;
    }
  }

  let barChart = $derived(rawData ? buildChart('bar') : '');
  let lineChart = $derived(rawData ? buildChart('line') : '');

  // ---- Derived request lists ----
  const pending = $derived(requests.filter(r => r.status === 'pending'));
  const inProgress = $derived(requests.filter(r => r.status === 'approved' || r.status === 'processing' || r.status === 'analyst_rejected'));
  const awaitingFinalReview = $derived(requests.filter(r => r.status === 'analyst_submitted'));
  const delivered = $derived(requests.filter(r => r.status === 'delivered'));
  const approvedForAnalyst = $derived(requests.filter(r => r.status === 'approved' || r.status === 'processing'));
  const submittedByAnalyst = $derived(requests.filter(r => r.status === 'analyst_submitted'));
  const rejectedByAnalyst = $derived(requests.filter(r => r.status === 'analyst_rejected'));
  const myRequests = $derived(requests);

  const analystUsers = $derived(allUsers.filter(u => u.role_name === 'data_analyst'));

  const statusBadge = (s: string) => {
    const map: Record<string, string> = {
      pending: 'badge-pending', approved: 'badge-approved', rejected: 'badge-rejected',
      processing: 'badge-processing', analyst_submitted: 'badge-submitted',
      analyst_rejected: 'badge-analyst-rejected', delivered: 'badge-delivered'
    };
    return map[s] || '';
  };

  const formatProvidedBy = (req: any) => {
    const raw = req?.provided_by_names;
    if (!raw) return '—';
    if (Array.isArray(raw) && raw.length) return raw.join(', ');
    if (typeof raw !== 'string') return '—';
    try {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length) return parsed.join(', ');
      return '—';
    } catch {
      return '—';
    }
  };
</script>

<svelte:head><title>RUSA IMS — Data Services</title></svelte:head>

<PageShell title="Data Services" subtitle="Manage data requests and analysis pipeline">

  {#if role === 'the_statistician'}
    <!-- ===== STATISTICIAN VIEW ===== -->
    <div class="tabs-bar">
      <button class="tab" class:active={activeTab==='pending'} onclick={() => activeTab='pending'}>Pending Review <span class="badge">{pending.length}</span></button>
      <button class="tab" class:active={activeTab==='inprogress'} onclick={() => activeTab='inprogress'}>In Progress <span class="badge">{inProgress.length}</span></button>
      <button class="tab" class:active={activeTab==='awaiting'} onclick={() => activeTab='awaiting'}>Awaiting Final Review <span class="badge">{awaitingFinalReview.length}</span></button>
      <button class="tab" class:active={activeTab==='delivered'} onclick={() => activeTab='delivered'}>Delivered <span class="badge">{delivered.length}</span></button>
    </div>

    {#if activeTab === 'pending'}
      <div class="section-bar"><h2 class="section-title">Pending Review</h2></div>
      {#if pending.length === 0}<p class="empty">No pending requests</p>{/if}
      {#each pending as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class="req-meta">{req.data_type || ''} · {req.created_at ? new Date(req.created_at).toLocaleDateString() : ''}</span>
            <span class="req-desc">{req.description || ''}</span>
          </div>
          <div class="req-actions">
            <button class="btn-approve" onclick={() => openReview(req)}>Review</button>
          </div>
        </div>
      {/each}

    {:else if activeTab === 'inprogress'}
      <div class="section-bar"><h2 class="section-title">In Progress</h2></div>
      {#if inProgress.length === 0}<p class="empty">No requests in progress</p>{/if}
      {#each inProgress as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class={`status-badge ${statusBadge(req.status)}`}>{req.status}</span>
            {#if req.status === 'analyst_rejected'}
              <span class="req-meta">Rejection reason: {req.analyst_rejection_reason || '—'}</span>
            {/if}
          </div>
        </div>
      {/each}

    {:else if activeTab === 'awaiting'}
      <div class="section-bar"><h2 class="section-title">Awaiting Final Review</h2></div>
      {#if awaitingFinalReview.length === 0}<p class="empty">No requests awaiting review</p>{/if}
      {#each awaitingFinalReview as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class="req-meta">Request ID: {req.id}</span>
            <span class="req-meta">Date of response: {req.response_submitted_at ? new Date(req.response_submitted_at).toLocaleDateString() : '—'}</span>
            <span class="req-meta">Status: {req.response_status ? req.response_status.toUpperCase() : '—'}</span>
            <span class="req-meta">Provided by: {formatProvidedBy(req)}</span>
            {#if req.response_status === 'rejected'}
              <span class="req-meta">Explanation: {req.response_explanation || '—'}</span>
            {/if}
            <MarkdownOutput markdown={req.response_markdown || req.response_data || ''} />
            <div class="req-actions" style="margin-top:0.5rem">
              <button class="btn-secondary" onclick={() => openAttachments(req)}>Attachments</button>
            </div>
          </div>
          <div class="req-actions">
            <button class="btn-primary" onclick={() => deliver(req)}>Deliver to Requester</button>
          </div>
        </div>
      {/each}

    {:else if activeTab === 'delivered'}
      <div class="section-bar"><h2 class="section-title">Delivered</h2></div>
      {#if delivered.length === 0}<p class="empty">No delivered requests</p>{/if}
      {#each delivered as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class="req-meta">Delivered: {req.delivered_at ? new Date(req.delivered_at).toLocaleDateString() : '—'}</span>
          </div>
        </div>
      {/each}
    {/if}

  {:else if role === 'data_analyst'}
    <!-- ===== DATA ANALYST VIEW ===== -->
    <div class="tabs-bar">
      <button class="tab" class:active={activeTab==='approved'} onclick={() => activeTab='approved'}>Approved Requests <span class="badge">{approvedForAnalyst.length}</span></button>
      <button class="tab" class:active={activeTab==='analyze'} onclick={() => activeTab='analyze'}>Analyze Data</button>
      <button class="tab" class:active={activeTab==='submit'} onclick={() => activeTab='submit'}>Submit Results <span class="badge">{submittedByAnalyst.length}</span></button>
    </div>

    {#if activeTab === 'approved'}
      <div class="section-bar"><h2 class="section-title">Approved Requests</h2></div>
      {#if approvedForAnalyst.length === 0}<p class="empty">No approved requests assigned to you</p>{/if}
      {#each approvedForAnalyst as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class={`status-badge ${statusBadge(req.status)}`}>{req.status}</span>
            <span class="req-meta">{req.data_type || ''} · {req.created_at ? new Date(req.created_at).toLocaleDateString() : ''}</span>
            <span class="req-desc">{req.description || ''}</span>
          </div>
          <div class="req-actions">
            {#if req.status === 'approved'}
              <button class="btn-secondary" onclick={() => startProcessing(req)}>Start Processing</button>
            {/if}
            <button class="btn-analyze" onclick={() => { activeRequest = req; activeTab = 'analyze'; }}>Analyze</button>
            <button class="btn-primary" onclick={() => openSubmit(req)}>Submit Results</button>
            <button class="btn-danger" onclick={() => openRejectAsResponse(req)}>Reject</button>
          </div>
        </div>
      {/each}

    {:else if activeTab === 'analyze'}
      <div class="section-bar">
        <h2 class="section-title">{activeRequest ? activeRequest.title : 'Select a request from Approved Requests'}</h2>
        {#if activeRequest}
          <button class="btn-primary" onclick={() => openSubmitWithData(activeRequest, analysisResults.join('\n\n'))}>Compose Response</button>
        {/if}
      </div>
      {#if activeRequest}
        <p class="req-desc-inline">{activeRequest.description || ''}</p>
        <div class="analysis-workspace">
          <div class="analysis-left">
            <label class="field-label" for="raw-data-input">Raw Data (CSV rows, one per line)</label>
            <textarea id="raw-data-input" class="field-input data-input" bind:value={rawData} rows={8} placeholder="col1,col2,col3&#10;1,2,3&#10;4,5,6"></textarea>
            <div class="tool-row">
              <input class="field-input tool-input" bind:value={filterCol} placeholder="Col index (0-based)" />
              <input class="field-input tool-input" bind:value={filterVal} placeholder="Value to filter" />
              <button class="btn-tool" onclick={runFilter}>Filter</button>
              <button class="btn-tool" onclick={runMean}>Mean</button>
              <button class="btn-tool" onclick={runMedian}>Median</button>
              <button class="btn-tool" onclick={runMode}>Mode</button>
              <button class="btn-tool" onclick={runVariance}>Variance</button>
              <button class="btn-tool" onclick={runMinMax}>Min/Max</button>
            </div>
            <div class="tool-row">
              <input class="field-input tool-input" bind:value={pivotGroupCol} placeholder="Group col index" />
              <input class="field-input tool-input" bind:value={pivotAggCol} placeholder="Aggregate col index" />
              <button class="btn-tool" onclick={runPivot}>Pivot Table</button>
            </div>
            {#if barChart}
              <div class="charts-row">
                <div>
                  <div class="chart-label">Bar Chart (col {filterCol})</div>
                  {@html barChart}
                </div>
                <div>
                  <div class="chart-label">Line Chart (col {filterCol})</div>
                  {@html lineChart}
                </div>
              </div>
            {/if}
          </div>
          <div class="analysis-right">
            <div class="results-header">
              Results
              <button class="btn-clear" onclick={() => analysisResults = []}>Clear</button>
            </div>
            {#if analysisResults.length === 0}
              <p class="empty">Run an analysis tool to see results</p>
            {:else}
              {#each analysisResults as r}
                <pre class="result-entry">{r}</pre>
              {/each}
            {/if}
          </div>
        </div>
      {:else}
        <p class="empty">Go to "Approved Requests" and click Analyze on a request</p>
      {/if}

    {:else if activeTab === 'submit'}
      <div class="section-bar"><h2 class="section-title">Submitted Results</h2></div>
      {#if submittedByAnalyst.length === 0}<p class="empty">No submissions yet</p>{/if}
      {#each submittedByAnalyst as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class={`status-badge ${statusBadge(req.status)}`}>{req.status}</span>
            <span class="req-meta">Submitted response, awaiting statistician delivery</span>
          </div>
        </div>
      {/each}
      {#if rejectedByAnalyst.length > 0}
        <div class="section-bar" style="margin-top:1.5rem"><h2 class="section-title">Rejected by You</h2></div>
        {#each rejectedByAnalyst as req}
          <div class="req-card">
            <div class="req-info">
              <span class="req-title">{req.title}</span>
              <span class={`status-badge ${statusBadge(req.status)}`}>{req.status}</span>
              <span class="req-meta">Rejection reason: {req.analyst_rejection_reason || '—'}</span>
            </div>
          </div>
        {/each}
      {/if}
    {/if}

  {:else}
    <!-- ===== ALL OTHER ROLES VIEW ===== -->
    <div class="section-bar">
      <h2 class="section-title">My Data Requests</h2>
      <button class="btn-primary" onclick={() => createOpen = true}>+ New Request</button>
    </div>
    {#if loading}
      <p class="loading">Loading...</p>
    {:else if myRequests.length === 0}
      <p class="empty">No requests submitted yet</p>
    {:else}
      {#each myRequests as req}
        <div class="req-card">
          <div class="req-info">
            <span class="req-title">{req.title}</span>
            <span class={`status-badge ${statusBadge(req.status)}`}>{req.status}</span>
            <span class="req-meta">{req.data_type || ''} · {req.created_at ? new Date(req.created_at).toLocaleDateString() : ''}</span>
            {#if req.status === 'delivered'}
              <span class="req-meta">Date of response: {req.response_submitted_at ? new Date(req.response_submitted_at).toLocaleDateString() : '—'}</span>
              <span class="req-meta">Response status: {req.response_status ? req.response_status.toUpperCase() : '—'}</span>
              <span class="req-meta">Provided by: {formatProvidedBy(req)}</span>
              <span class="req-meta">Final review: {req.delivered_at ? new Date(req.delivered_at).toLocaleDateString() : '—'}</span>
              {#if req.response_status === 'rejected'}
                <span class="req-meta">Explanation: {req.response_explanation || '—'}</span>
              {/if}
              <MarkdownOutput markdown={req.response_markdown || req.response_data || ''} />
              <div class="req-actions" style="margin-top:0.5rem">
                <button class="btn-secondary" onclick={() => openAttachments(req)}>Attachments</button>
                {#if !req.requester_acknowledged_at}
                  <button class="btn-primary" onclick={() => acknowledge(req)}>Acknowledge Receipt</button>
                {:else}
                  <span class="req-meta">Acknowledged: {new Date(req.requester_acknowledged_at).toLocaleDateString()}</span>
                {/if}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  {/if}

</PageShell>

<!-- New Request Modal -->
<Modal bind:open={createOpen} title="Submit Data Request">
  <div class="form">
    <Field label="Location" bind:value={requesterLocation} required />
    <Field label="Tel/Fax" bind:value={requesterTelFax} required />
    <Field label="Department" bind:value={requesterDepartment} required />
    <Field label="Department Email" bind:value={requesterDepartmentEmail} required />

    <Field label="Date" value={new Date().toLocaleDateString()} disabled />
    <Field label="Title" bind:value={title} required />
    <Field label="Data Type" bind:value={dataType} placeholder="e.g. telemetry, environmental" />

    <div class="items">
      <div class="items-head">
        <div class="items-title">Requested Data (Itemized)<span class="required">*</span></div>
        <button class="btn-secondary" type="button" onclick={addRequestedItem}>+ Add Item</button>
      </div>
      {#each requestedItems as item, i}
        <div class="item-row">
          <input class="item-input" bind:value={requestedItems[i]} placeholder="e.g. Ship data, Time logs, Participants" />
          <button class="btn-danger" type="button" onclick={() => removeRequestedItem(i)} disabled={requestedItems.length<=1}>Remove</button>
        </div>
      {/each}
    </div>

    <Field label="Reason of Request" type="textarea" bind:value={reasonOfRequest} required />
    <Field label="Additional Context" type="textarea" bind:value={additionalContext} />

    <Field label="Requested By" value={$session?.full_name ?? ''} disabled />
    <Field label="Signature" value={$session?.full_name ?? ''} disabled />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => createOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={create}>Submit</button>
    </div>
  </div>
</Modal>

<!-- Statistician Review Modal -->
<Modal bind:open={reviewOpen} title="Review: {selectedReq?.title}">
  <div class="form">
    <p class="req-meta">Status: <strong>{selectedReq?.status}</strong></p>
    <p class="req-desc">{selectedReq?.description || ''}</p>
    <Field label="Review Notes" type="textarea" bind:value={reviewNotes} />
    {#if analystUsers.length > 0}
      <div class="field">
        <label class="field-label" for="assign-analyst">Assign to Analyst (optional)</label>
        <select id="assign-analyst" class="field-input" bind:value={assignedTo}>
          <option value="">— Not assigned —</option>
          {#each analystUsers as u}
            <option value={u.id}>{u.full_name || u.username}</option>
          {/each}
        </select>
      </div>
    {/if}
    <div class="form-actions">
      <button class="btn-danger" onclick={() => submitReview('rejected')}>Reject</button>
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Close</button>
      <button class="btn-approve" onclick={() => submitReview('approved')}>Approve</button>
    </div>
  </div>
</Modal>

<!-- Analyst Submit Modal -->
<Modal bind:open={submitOpen} title="Submit Results: {submitReq?.title}">
  <div class="form">
    <p class="req-meta">Status: <strong>{submitReq?.status}</strong></p>
    <p class="req-desc">{submitReq?.description || ''}</p>
    <Field
      label="Response Status"
      type="select"
      bind:value={responseStatus}
      options={[
        { value: 'provided', label: 'PROVIDED' },
        { value: 'rejected', label: 'REJECTED' }
      ]}
      required
    />
    {#if responseStatus === 'rejected'}
      <Field label="Explanation" type="textarea" bind:value={responseExplanation} rows={4} required />
    {/if}
    <Field label="Provided Data Output (Markdown supported)" type="textarea" bind:value={responseMarkdown} rows={10} required={responseStatus==='provided'} />

    <div class="field">
      <label class="field-label">Attachments</label>
      <input class="field-input" type="file" multiple onchange={onFilesSelected} />
      {#if selectedFiles.length}
        <div class="file-list">
          {#each selectedFiles as f}
            <div class="file-row">{f.name}</div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="field">
      <label class="field-label">Provided By<span class="required">*</span></label>
      <div class="checkboxes">
        {#each analystUsers as u}
          <label class="check">
            <input type="checkbox" checked={providedBy.includes(u.id)} onchange={() => toggleProvidedBy(u.id)} />
            <span>{u.full_name || u.username}</span>
          </label>
        {/each}
      </div>
    </div>

    <Field label="Analyst Notes" type="textarea" bind:value={analystNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => submitOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={doAnalystSubmit}>Submit to Statistician</button>
    </div>
  </div>
</Modal>

<!-- Attachments Modal -->
<Modal bind:open={attachmentsOpen} title="Attachments: {attachmentsReq?.title}">
  <div class="form">
    {#if attachments.length === 0}
      <p class="empty">No attachments</p>
    {:else}
      {#each attachments as a}
        <div class="att-row">
          <span class="att-name">{a.filename}</span>
          <button class="btn-secondary" onclick={() => downloadAttachment(a.id)}>Download</button>
        </div>
      {/each}
    {/if}
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => attachmentsOpen = false}>Close</button>
    </div>
  </div>
</Modal>

<style>
  .tabs-bar { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; padding: 0.6rem 1rem; text-transform: uppercase; letter-spacing: 0.08em; display: flex; align-items: center; gap: 0.4rem; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .badge { background: #1e2d4a; color: #8fa3cc; font-size: 0.65rem; border-radius: 10px; padding: 1px 6px; }
  .tab.active .badge { background: rgba(0,212,255,0.15); color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading, .empty { color: #4a5d82; padding: 1.5rem 0; font-size: 0.9rem; }
  .req-card { display: flex; align-items: flex-start; justify-content: space-between; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem 1.25rem; margin-bottom: 0.75rem; gap: 1rem; }
  .req-info { display: flex; flex-direction: column; gap: 0.25rem; flex: 1; }
  .req-title { font-size: 0.95rem; color: #e8eeff; font-weight: 500; }
  .req-meta { font-size: 0.78rem; color: #8fa3cc; }
  .req-desc { font-size: 0.82rem; color: #6a7f9a; }
  .req-desc-inline { font-size: 0.85rem; color: #8fa3cc; margin-bottom: 1rem; }
  .response-preview { font-family: 'Space Mono', monospace; font-size: 0.75rem; color: #6a7f9a; white-space: pre-wrap; max-height: 60px; overflow: hidden; }
  .req-actions { display: flex; gap: 0.5rem; flex-shrink: 0; flex-wrap: wrap; align-items: flex-start; }
  .status-badge { display: inline-block; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.06em; }
  .badge-pending { background: rgba(255,170,0,0.12); color: #ffaa00; }
  .badge-approved { background: rgba(0,212,255,0.1); color: #00d4ff; }
  .badge-rejected { background: rgba(255,68,102,0.1); color: #ff4466; }
  .badge-processing { background: rgba(61,127,255,0.1); color: #3d7fff; }
  .badge-submitted { background: rgba(180,130,255,0.1); color: #b482ff; }
  .badge-analyst-rejected { background: rgba(255,140,0,0.12); color: #ff8c00; }
  .badge-delivered { background: rgba(0,200,100,0.1); color: #00c864; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .field-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.625rem 0.875rem; width: 100%; box-sizing: border-box; }
  .field-input:focus { outline: none; border-color: #3d7fff; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.5rem 1rem; white-space: nowrap; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.82rem; padding: 0.5rem 1rem; white-space: nowrap; }
  .btn-approve { background: none; border: 1px solid #00c864; border-radius: 4px; color: #00c864; cursor: pointer; font-size: 0.82rem; padding: 0.5rem 1rem; white-space: nowrap; }
  .btn-danger { background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.82rem; padding: 0.5rem 1rem; white-space: nowrap; }
  .btn-analyze { background: none; border: 1px solid #b482ff; border-radius: 4px; color: #b482ff; cursor: pointer; font-size: 0.82rem; padding: 0.5rem 1rem; white-space: nowrap; }
  .items { display: flex; flex-direction: column; gap: 0.75rem; }
  .items-head { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; }
  .items-title { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .required { color: #ff4466; margin-left: 2px; }
  .item-row { display: grid; grid-template-columns: 1fr auto; gap: 0.75rem; align-items: center; }
  .item-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.625rem 0.875rem; width: 100%; box-sizing: border-box; }
  .item-input:focus { outline: none; border-color: #3d7fff; }
  .file-list { margin-top: 0.5rem; display: flex; flex-direction: column; gap: 0.25rem; }
  .file-row { font-size: 0.82rem; color: #8fa3cc; }
  .checkboxes { display: flex; flex-direction: column; gap: 0.5rem; }
  .check { display: flex; align-items: center; gap: 0.5rem; font-size: 0.85rem; color: #8fa3cc; }
  .att-row { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.5rem 0; border-bottom: 1px solid #0d1528; }
  .att-row:last-child { border-bottom: none; }
  .att-name { color: #e8eeff; font-size: 0.9rem; }
  .analysis-workspace { display: grid; grid-template-columns: 1fr 340px; gap: 1.5rem; }
  .analysis-left { display: flex; flex-direction: column; gap: 0.75rem; }
  .data-input { font-family: 'Space Mono', monospace; font-size: 0.8rem; resize: vertical; }
  .tool-row { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
  .tool-input { width: 130px; }
  .btn-tool { background: #1e2d4a; border: none; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; padding: 0.4rem 0.7rem; white-space: nowrap; }
  .btn-tool:hover { background: #2a3d5a; color: #e8eeff; }
  .charts-row { display: flex; gap: 1rem; flex-wrap: wrap; margin-top: 0.5rem; }
  .chart-label { font-size: 0.7rem; color: #8fa3cc; font-family: 'Space Mono', monospace; margin-bottom: 0.25rem; }
  .analysis-right { background: #080d1a; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem; overflow-y: auto; max-height: 500px; }
  .results-header { display: flex; align-items: center; justify-content: space-between; font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; margin-bottom: 0.75rem; }
  .btn-clear { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #4a5d82; cursor: pointer; font-size: 0.7rem; padding: 2px 8px; }
  .result-entry { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Space Mono', monospace; font-size: 0.75rem; margin-bottom: 0.5rem; padding: 0.625rem 0.75rem; white-space: pre-wrap; word-break: break-all; }
</style>
