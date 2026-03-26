<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { dataApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';

  let title = $state('');
  let dataType = $state('');
  let additionalContext = $state('');

  let requesterLocation = $state('');
  let requesterTelFax = $state('N/A');
  let requesterDepartment = $state('');
  let requesterDepartmentEmail = $state('');

  let requestedItems = $state<string[]>(['']);
  let reasonOfRequest = $state('');

  $effect(() => {
    const s = $session;
    if (!s) return;
    if (!requesterLocation) requesterLocation = s.location ?? '';
    if (requesterTelFax === 'N/A' || !requesterTelFax) requesterTelFax = s.tel_fax ?? 'N/A';
    if (!requesterDepartment) requesterDepartment = s.department ?? s.role_display_name ?? '';
    if (!requesterDepartmentEmail) requesterDepartmentEmail = s.department_email ?? s.email ?? '';
  });

  let loading = $state(false);

  function addRequestedItem() {
    requestedItems = [...requestedItems, ''];
  }

  function removeRequestedItem(i: number) {
    if (requestedItems.length <= 1) return;
    requestedItems = requestedItems.filter((_, idx) => idx !== i);
  }

  async function submit() {
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
    loading = true;
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
      showToast('Data request submitted successfully', 'success');
      title = '';
      dataType = '';
      additionalContext = '';
      requestedItems = [''];
      reasonOfRequest = '';
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
    loading = false;
  }
</script>


<svelte:head><title>RUSA IMS — Request Data</title></svelte:head>

<PageShell title="Request Data" subtitle="Submit a data access request to the data services team">
  <div class="form-card">
    <div class="section">
      <div class="section-title">Header / Department Info</div>
      <Field label="Location" bind:value={requesterLocation} required />
      <Field label="Tel/Fax" bind:value={requesterTelFax} required />
      <Field label="Department" bind:value={requesterDepartment} required />
      <Field label="Department Email" bind:value={requesterDepartmentEmail} required />
    </div>

    <div class="section">
      <div class="section-title">Request</div>
      <Field label="Date" value={new Date().toLocaleDateString()} disabled />
      <Field label="Request Title" bind:value={title} required />
      <Field label="Data Type" bind:value={dataType} placeholder="e.g. telemetry, environmental, biological" />

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

      <Field label="Reason of Request" type="textarea" bind:value={reasonOfRequest} rows={4} required />
      <Field label="Additional Context" type="textarea" bind:value={additionalContext} rows={4} />

      <Field label="Requested By" value={$session?.full_name ?? ''} disabled />
      <Field label="Signature" value={$session?.full_name ?? ''} disabled />
    </div>
    <div class="form-actions">
      <button class="btn-primary" onclick={submit} disabled={loading}>
        {loading ? 'Submitting...' : 'Submit Request'}
      </button>
    </div>
  </div>
</PageShell>

<style>
  .form-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; max-width: 600px; display: flex; flex-direction: column; gap: 1.25rem; }
  .section { display: flex; flex-direction: column; gap: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .items { display: flex; flex-direction: column; gap: 0.75rem; }
  .items-head { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; }
  .items-title { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .required { color: #ff4466; margin-left: 2px; }
  .item-row { display: grid; grid-template-columns: 1fr auto; gap: 0.75rem; align-items: center; }
  .item-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.625rem 0.875rem; width: 100%; box-sizing: border-box; }
  .item-input:focus { outline: none; border-color: #3d7fff; box-shadow: 0 0 0 2px rgba(61,127,255,0.15); }
  .form-actions { display: flex; justify-content: flex-end; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.82rem; padding: 0.35rem 0.65rem; white-space: nowrap; }
  .btn-danger { background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.82rem; padding: 0.35rem 0.65rem; white-space: nowrap; }
  .btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
