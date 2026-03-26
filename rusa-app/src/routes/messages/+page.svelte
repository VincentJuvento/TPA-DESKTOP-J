<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { messageApi, userApi, researchTaskApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let activeTab = $state<'inbox' | 'sent'>('inbox');
  let inbox: any[] = $state([]);
  let sent: any[] = $state([]);
  let selectedMessage: any = $state(null);
  let composeOpen = $state(false);
  let loading = $state(false);

  // Compose state
  let allUsers: any[] = $state([]);
  let toSelected: any[] = $state([]);
  let ccSelected: any[] = $state([]);
  let bccSelected: any[] = $state([]);
  let toSearch = $state('');
  let ccSearch = $state('');
  let bccSearch = $state('');
  let subject = $state('');
  let body = $state('');

  let unlisten: (() => void) | null = null;

  // Proxy flow: Convert inbox message to research task (Observer/Artificer/Taskmaster)
  const canConvertToTask = $derived(
    canPerform($session, 'the_observer') ||
    canPerform($session, 'the_artificer') ||
    canPerform($session, 'the_taskmaster')
  );
  let convertTaskOpen = $state(false);
  let ctTitle = $state('');
  let ctDesc = $state('');
  let ctAssigneeSelected: any = $state(null);
  let ctDue = $state('');

  async function openConvertToTask(msg: any) {
    const s = $session; if (!s) return;
    ctSourceMsgId = msg.id;
    ctTitle = msg.subject || '';
    ctDesc = '';
    ctAssigneeSelected = null;
    ctDue = '';
    if (allUsers.length === 0) {
      try { allUsers = await userApi.getAll(s.token); } catch {}
    }
    convertTaskOpen = true;
  }

  async function submitConvertToTask() {
    const s = $session; if (!s) return;
    if (!ctTitle.trim()) { showToast('Title required', 'error'); return; }
    try {
      await researchTaskApi.assign(
        s.token,
        ctTitle,
        ctDesc || undefined,
        ctAssigneeSelected?.id || undefined,
        ctSourceMsgId || undefined,
        ctDue ? new Date(ctDue).toISOString() : undefined,
      );
      showToast('Help request converted to research task', 'success');
      convertTaskOpen = false;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  onMount(async () => {
    const s = $session;
    if (!s) return;
    await loadMessages();
    try {
      unlisten = await listen('new_message', () => {
        loadMessages();
        showToast('New message received!', 'info');
      });
    } catch {}
  });

  onDestroy(() => { unlisten?.(); });

  async function loadMessages() {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      [inbox, sent] = await Promise.all([
        messageApi.getInbox(s.token),
        messageApi.getSent(s.token)
      ]);
    } catch (e: any) {
      showToast('Failed to load messages: ' + e, 'error');
    }
    loading = false;
  }

  async function openCompose() {
    const s = $session; if (!s) return;
    subject = ''; body = ''; toSelected = []; ccSelected = []; bccSelected = [];
    toSearch = ''; ccSearch = ''; bccSearch = '';
    try {
      allUsers = await userApi.getAll(s.token);
    } catch {
      allUsers = [];
    }
    composeOpen = true;
  }

  async function openMessage(msg: any) {
    const s = $session;
    if (!s) return;
    try {
      selectedMessage = await messageApi.getMessage(s.token, msg.id);
      await messageApi.markRead(s.token, msg.id);
      await loadMessages();
    } catch {}
  }

  async function sendMessage() {
    const s = $session;
    if (!s) return;
    if (!subject || !body) { showToast('Subject and body required', 'error'); return; }
    if (toSelected.length === 0 && ccSelected.length === 0 && bccSelected.length === 0) {
      showToast('Please select at least one recipient', 'error'); return;
    }
    try {
      await messageApi.send(
        s.token,
        toSelected.map(u => u.id),
        ccSelected.map(u => u.id),
        bccSelected.map(u => u.id),
        subject,
        body
      );
      showToast('Message sent!', 'success');
      composeOpen = false;
      await loadMessages();
    } catch (e: any) {
      showToast('Failed to send: ' + e, 'error');
    }
  }

  async function recallMessage(msg: any) {
    const s = $session;
    if (!s) return;
    try {
      await messageApi.recall(s.token, msg.id);
      showToast('Message recalled', 'success');
      selectedMessage = null;
      await loadMessages();
    } catch (e: any) {
      showToast('Failed to recall: ' + e, 'error');
    }
  }

  function addRecipient(list: any[], user: any): any[] {
    if (list.find(u => u.id === user.id)) return list;
    return [...list, user];
  }
  function removeRecipient(list: any[], userId: string): any[] {
    return list.filter(u => u.id !== userId);
  }

  function filteredUsers(search: string, excluded: any[]): any[] {
    const q = search.toLowerCase();
    return allUsers.filter(u =>
      !excluded.find(e => e.id === u.id) &&
      (u.username?.toLowerCase().includes(q) || u.full_name?.toLowerCase().includes(q))
    ).slice(0, 8);
  }

  let toSuggestions = $derived(toSearch.length > 0 ? filteredUsers(toSearch, [...toSelected, ...ccSelected, ...bccSelected]) : []);
  let ccSuggestions = $derived(ccSearch.length > 0 ? filteredUsers(ccSearch, [...toSelected, ...ccSelected, ...bccSelected]) : []);
  let bccSuggestions = $derived(bccSearch.length > 0 ? filteredUsers(bccSearch, [...toSelected, ...ccSelected, ...bccSelected]) : []);

  let currentList = $derived(activeTab === 'inbox' ? inbox : sent);
  let unread = $derived(inbox.filter(m => !m.read_at).length);
</script>

<svelte:head><title>RUSA IMS — Messages</title></svelte:head>

<PageShell title="Internal Messages" subtitle="Secure internal communications system">
  <div class="messages-layout">
    <div class="messages-sidebar">
      <div class="messages-tabs">
        <button class="tab" class:active={activeTab==='inbox'} onclick={() => activeTab='inbox'}>
          Inbox {#if unread > 0}<span class="badge">{unread}</span>{/if}
        </button>
        <button class="tab" class:active={activeTab==='sent'} onclick={() => activeTab='sent'}>Sent</button>
      </div>
      <button class="compose-btn" onclick={openCompose}>+ Compose</button>
      <div class="message-list">
        {#if loading}
          <p class="list-empty">Loading...</p>
        {:else if currentList.length === 0}
          <p class="list-empty">No messages</p>
        {:else}
          {#each currentList as msg}
            <button class="msg-item" class:unread={!msg.read_at && activeTab==='inbox'} class:selected={selectedMessage?.id===msg.id} class:broadcast={msg.is_broadcast} onclick={() => openMessage(msg)}>
              {#if msg.is_broadcast}
                <div class="broadcast-flag">HIGH PRIORITY</div>
              {/if}
              <div class="msg-item-from">{msg.is_broadcast ? msg.broadcast_sender || 'SYSTEM' : (msg.from_name || msg.from_username || 'Unknown')}</div>
              <div class="msg-item-subject">{msg.subject}</div>
              <div class="msg-item-date">{msg.sent_at ? new Date(msg.sent_at).toLocaleDateString() : '—'}</div>
            </button>
          {/each}
        {/if}
      </div>
    </div>

    <div class="message-view">
      {#if selectedMessage}
        <div class="msg-header">
          {#if selectedMessage.is_broadcast}
            <div class="broadcast-banner">SYSTEM BROADCAST - HIGH PRIORITY</div>
          {/if}
          <h2 class="msg-subject-title">{selectedMessage.subject}</h2>
          <div class="msg-meta">
            <span>From: <strong>{selectedMessage.is_broadcast ? selectedMessage.broadcast_sender || 'SYSTEM' : (selectedMessage.from_name || selectedMessage.from_username)}</strong></span>
            <span>Sent: {selectedMessage.sent_at ? new Date(selectedMessage.sent_at).toLocaleString() : '—'}</span>
          </div>
          {#if selectedMessage.recalled_at}
            <div class="recalled-banner">⚠ This message has been recalled</div>
          {/if}
          {#if activeTab === 'sent' && !selectedMessage.recalled_at}
            <button class="recall-btn" onclick={() => recallMessage(selectedMessage)}>Recall Message</button>
          {/if}
          {#if canConvertToTask && activeTab === 'inbox' && !selectedMessage.recalled_at}
            <button class="convert-btn" onclick={() => openConvertToTask(selectedMessage)}>🔁 Convert to Research Task</button>
          {/if}
        </div>
        <div class="msg-body">{selectedMessage.body}</div>
      {:else}
        <div class="msg-empty">Select a message to read</div>
      {/if}
    </div>
  </div>
</PageShell>

<Modal bind:open={composeOpen} title="New Message">
  <div class="compose-form">
    <!-- To field -->
    <div class="field">
      <label class="field-label" for="compose-to-search">To</label>
      <div class="recipient-tokens">
        {#each toSelected as u}
          <span class="token">{u.full_name || u.username}<button class="token-remove" onclick={() => toSelected = removeRecipient(toSelected, u.id)}>×</button></span>
        {/each}
        <div class="autocomplete-wrap">
          <input id="compose-to-search" class="field-input inline-input" bind:value={toSearch} placeholder="Search by name or username…" />
          {#if toSuggestions.length > 0}
            <div class="suggestions">
              {#each toSuggestions as u}
                <button class="suggestion-item" onclick={() => { toSelected = addRecipient(toSelected, u); toSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
    <!-- CC field -->
    <div class="field">
      <label class="field-label" for="compose-cc-search">CC (optional)</label>
      <div class="recipient-tokens">
        {#each ccSelected as u}
          <span class="token">{u.full_name || u.username}<button class="token-remove" onclick={() => ccSelected = removeRecipient(ccSelected, u.id)}>×</button></span>
        {/each}
        <div class="autocomplete-wrap">
          <input id="compose-cc-search" class="field-input inline-input" bind:value={ccSearch} placeholder="Search…" />
          {#if ccSuggestions.length > 0}
            <div class="suggestions">
              {#each ccSuggestions as u}
                <button class="suggestion-item" onclick={() => { ccSelected = addRecipient(ccSelected, u); ccSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
    <!-- BCC field -->
    <div class="field">
      <label class="field-label" for="compose-bcc-search">BCC (optional)</label>
      <div class="recipient-tokens">
        {#each bccSelected as u}
          <span class="token">{u.full_name || u.username}<button class="token-remove" onclick={() => bccSelected = removeRecipient(bccSelected, u.id)}>×</button></span>
        {/each}
        <div class="autocomplete-wrap">
          <input id="compose-bcc-search" class="field-input inline-input" bind:value={bccSearch} placeholder="Search…" />
          {#if bccSuggestions.length > 0}
            <div class="suggestions">
              {#each bccSuggestions as u}
                <button class="suggestion-item" onclick={() => { bccSelected = addRecipient(bccSelected, u); bccSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
    <div class="field">
      <label class="field-label" for="compose-subject">Subject *</label>
      <input id="compose-subject" class="field-input" bind:value={subject} placeholder="Message subject" />
    </div>
    <div class="field">
      <label class="field-label" for="compose-body">Body *</label>
      <textarea id="compose-body" class="field-input" bind:value={body} rows={6} placeholder="Write your message..."></textarea>
    </div>
    <div class="compose-actions">
      <button class="btn-secondary" onclick={() => composeOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={sendMessage}>Send Message</button>
    </div>
  </div>
</Modal>

<Modal bind:open={convertTaskOpen} title="Convert Message to Research Task">
  <div class="convert-form">
    <p class="proxy-note">This message will be linked to the new research task as its source.</p>
    <div class="field">
      <label class="field-label" for="ct-title">Task Title *</label>
      <input id="ct-title" class="field-input" bind:value={ctTitle} placeholder="Task title…" />
    </div>
    <div class="field">
      <label class="field-label" for="ct-desc">Description</label>
      <textarea id="ct-desc" class="field-input" bind:value={ctDesc} placeholder="Optional description…" rows="3"></textarea>
    </div>
    <div class="field">
      <label class="field-label" for="ct-assignee">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={ctAssigneeSelected} placeholder="Search user…" />
    </div>
    <div class="field">
      <label class="field-label" for="ct-due">Due Date</label>
      <input id="ct-due" class="field-input" type="date" bind:value={ctDue} />
    </div>
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => convertTaskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitConvertToTask}>Create Task</button>
    </div>
  </div>
</Modal>

<style>
  .messages-layout { display: grid; grid-template-columns: 300px 1fr; gap: 0; height: calc(100vh - 180px); border: 1px solid #1e2d4a; border-radius: 8px; overflow: hidden; }
  .messages-sidebar { background: #0d1528; border-right: 1px solid #1e2d4a; display: flex; flex-direction: column; }
  .messages-tabs { display: flex; border-bottom: 1px solid #1e2d4a; }
  .tab { flex: 1; background: none; border: none; color: #8fa3cc; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 0.5rem; font-family: 'Space Mono', monospace; font-size: 0.7rem; padding: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom: 2px solid #00d4ff; }
  .badge { background: #ffaa00; color: #05070f; font-size: 0.65rem; border-radius: 10px; padding: 1px 6px; font-weight: 700; }
  .compose-btn { margin: 0.75rem; padding: 0.5rem; background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; font-weight: 700; letter-spacing: 0.08em; }
  .message-list { flex: 1; overflow-y: auto; }
  .list-empty { padding: 1.5rem; text-align: center; color: #4a5d82; font-size: 0.85rem; }
  .msg-item {
    display: flex; flex-direction: column; gap: 0.25rem;
    padding: 0.875rem 1rem; border: none; border-bottom: 1px solid #1e2d4a;
    background: transparent; color: #8fa3cc; text-align: left;
    cursor: pointer; transition: background 0.15s; position: relative;
    width: 100%;
  }
  .msg-item:hover { background: rgba(61,127,255,0.04); }
  .msg-item.selected { background: #1a2642; }
  .msg-item.unread .msg-item-from { color: #e8eeff; font-weight: 600; }
  .msg-item.unread .msg-item-subject { color: #e8eeff; }
  
  .msg-item.broadcast {
    background: rgba(255, 68, 102, 0.05);
    border-left: 3px solid #ff4466;
  }
  .msg-item.broadcast:hover { background: rgba(255, 68, 102, 0.1); }
  .msg-item.broadcast.selected { background: rgba(255, 68, 102, 0.15); }
  .msg-item.broadcast .msg-item-from { color: #ff4466; font-weight: 700; }
  
  .broadcast-flag {
    font-size: 0.6rem;
    font-family: 'Space Mono', monospace;
    color: #ff4466;
    background: rgba(255, 68, 102, 0.15);
    padding: 2px 6px;
    border-radius: 3px;
    align-self: flex-start;
    margin-bottom: 0.25rem;
    letter-spacing: 0.05em;
  }

  .msg-item-from { font-size: 0.85rem; color: #c8d8f0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .msg-item-subject { font-size: 0.9rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .msg-item-date { font-family: 'Space Mono', monospace; font-size: 0.7rem; color: #4a5d82; margin-top: 0.25rem; }
  .message-view { background: #080d1a; padding: 2rem; overflow-y: auto; }
  .msg-empty { color: #4a5d82; text-align: center; margin-top: 4rem; }
  .msg-header { border-bottom: 1px solid #1e2d4a; padding-bottom: 1.25rem; margin-bottom: 1.5rem; }
  .broadcast-banner { background: #ff4466; color: #fff; font-weight: 700; padding: 0.5rem 1rem; border-radius: 4px; margin-bottom: 1rem; text-align: center; letter-spacing: 0.05em; font-size: 0.85rem; }
  .msg-subject-title { font-family: 'Space Mono', monospace; font-size: 1rem; color: #e8eeff; margin-bottom: 0.75rem; }
  .msg-meta { display: flex; gap: 2rem; font-size: 0.82rem; color: #8fa3cc; }
  .recalled-banner { background: rgba(255,170,0,0.1); border: 1px solid rgba(255,170,0,0.3); border-radius: 4px; color: #ffaa00; font-size: 0.8rem; margin-top: 0.75rem; padding: 0.5rem 0.75rem; }
  .recall-btn { margin-top: 0.75rem; background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.8rem; padding: 0.375rem 0.75rem; }
  .msg-body { color: #e8eeff; font-size: 0.9rem; line-height: 1.7; white-space: pre-wrap; }
  .compose-form { display: flex; flex-direction: column; gap: 1rem; }
  .compose-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .field-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.625rem 0.875rem; width: 100%; box-sizing: border-box; }
  .field-input:focus { outline: none; border-color: #3d7fff; }
  textarea.field-input { resize: vertical; min-height: 120px; }
  .recipient-tokens { display: flex; flex-wrap: wrap; gap: 0.375rem; align-items: flex-start; background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.375rem 0.5rem; min-height: 40px; }
  .token { display: inline-flex; align-items: center; gap: 0.25rem; background: rgba(61,127,255,0.15); border: 1px solid rgba(61,127,255,0.3); border-radius: 12px; color: #8fa3cc; font-size: 0.8rem; padding: 2px 8px; }
  .token-remove { background: none; border: none; color: #8fa3cc; cursor: pointer; font-size: 0.9rem; line-height: 1; padding: 0 2px; }
  .token-remove:hover { color: #ff4466; }
  .autocomplete-wrap { position: relative; flex: 1; min-width: 160px; }
  .inline-input { border: none !important; background: transparent !important; padding: 2px 4px !important; width: 100%; font-size: 0.85rem; }
  .inline-input:focus { outline: none; }
  .suggestions { position: absolute; top: 100%; left: 0; right: 0; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; z-index: 100; max-height: 200px; overflow-y: auto; }
  .suggestion-item { display: block; width: 100%; background: none; border: none; border-bottom: 1px solid #1e2d4a; color: #e8eeff; cursor: pointer; font-size: 0.85rem; padding: 0.5rem 0.75rem; text-align: left; }
  .suggestion-item:last-child { border-bottom: none; }
  .suggestion-item:hover { background: rgba(61,127,255,0.1); }
  .suggestion-role { color: #4a5d82; font-size: 0.75rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .convert-btn { margin-top: 0.75rem; background: none; border: 1px solid #3d7fff; border-radius: 4px; color: #3d7fff; cursor: pointer; font-size: 0.8rem; padding: 0.375rem 0.75rem; }
  .convert-form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .proxy-note { background: rgba(61,127,255,0.07); border: 1px solid rgba(61,127,255,0.2); border-radius: 4px; color: #8fa3cc; font-size: 0.8rem; margin: 0; padding: 0.5rem 0.75rem; }
</style>
