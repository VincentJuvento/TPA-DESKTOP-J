<script lang="ts">
  let {
    users = [] as any[],
    selected = $bindable(null as any),
    placeholder = 'Search by name or username…',
  } = $props();

  let search = $state('');

  function filterUsers(q: string): any[] {
    const lower = q.toLowerCase();
    return users.filter((u: any) =>
      u.username?.toLowerCase().includes(lower) || u.full_name?.toLowerCase().includes(lower)
    ).slice(0, 8);
  }

  let suggestions = $derived(search.length > 0 ? filterUsers(search) : []);

  function selectUser(u: any) {
    selected = u;
    search = '';
  }

  function clear() {
    selected = null;
    search = '';
  }
</script>

<div class="assignee-autocomplete">
  {#if selected}
    <span class="token">
      {selected.full_name || selected.username}
      <button class="token-remove" onclick={clear}>×</button>
    </span>
  {:else}
    <div class="autocomplete-wrap">
      <input class="field-input" bind:value={search} {placeholder} />
      {#if suggestions.length > 0}
        <div class="suggestions">
          {#each suggestions as u}
            <button class="suggestion-item" onclick={() => selectUser(u)}>
              {u.full_name || u.username} <span class="suggestion-role">@{u.username}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .assignee-autocomplete { position: relative; }
  .autocomplete-wrap { position: relative; }
  .field-input {
    background: #05070f;
    border: 1px solid #1e2d4a;
    border-radius: 4px;
    color: #e8eeff;
    font-family: 'Outfit', sans-serif;
    font-size: 0.9rem;
    padding: 0.625rem 0.875rem;
    width: 100%;
    box-sizing: border-box;
  }
  .field-input:focus { outline: none; border-color: #3d7fff; box-shadow: 0 0 0 2px rgba(61,127,255,0.15); }
  .field-input::placeholder { color: #4a5d82; }
  .suggestions {
    position: absolute; top: 100%; left: 0; right: 0;
    background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px;
    z-index: 100; max-height: 200px; overflow-y: auto;
  }
  .suggestion-item {
    display: block; width: 100%; background: none; border: none;
    border-bottom: 1px solid #1e2d4a; color: #e8eeff; cursor: pointer;
    font-size: 0.85rem; padding: 0.5rem 0.75rem; text-align: left;
  }
  .suggestion-item:last-child { border-bottom: none; }
  .suggestion-item:hover { background: rgba(61,127,255,0.1); }
  .suggestion-role { color: #4a5d82; font-size: 0.75rem; }
  .token {
    display: inline-flex; align-items: center; gap: 0.25rem;
    background: rgba(61,127,255,0.15); border: 1px solid rgba(61,127,255,0.3);
    border-radius: 12px; color: #8fa3cc; font-size: 0.8rem; padding: 2px 8px;
  }
  .token-remove {
    background: none; border: none; color: #8fa3cc;
    cursor: pointer; font-size: 0.9rem; line-height: 1; padding: 0 2px;
  }
  .token-remove:hover { color: #ff4466; }
</style>
