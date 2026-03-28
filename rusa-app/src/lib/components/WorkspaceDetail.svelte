<script lang="ts">
  import type { Snippet } from 'svelte';

  type Tag = { label: string; className?: string };
  type Action = { label: string; className?: string; disabled?: boolean; onClick: () => void };
  type DetailTab = { key: string; label: string; count?: number };

  let {
    backLabel = 'Back',
    onBack = (() => {}) as () => void,
    tags = [] as Tag[],
    actions = [] as Action[],
    tabs = [] as DetailTab[],
    activeTab = '',
    onSelectTab = ((_: string) => {}) as (tabKey: string) => void,
    body,
  }: {
    backLabel?: string;
    onBack?: () => void;
    tags?: Tag[];
    actions?: Action[];
    tabs?: DetailTab[];
    activeTab?: string;
    onSelectTab?: (tabKey: string) => void;
    body?: Snippet<[]>;
  } = $props();
</script>

<div class="workspace-header">
  <div class="header-top">
    <button class="btn-back" onclick={onBack}>← {backLabel}</button>
    <div class="header-actions">
      {#each actions as action}
        <button class={action.className ?? 'btn-secondary'} onclick={action.onClick} disabled={action.disabled ?? false}>
          {action.label}
        </button>
      {/each}
    </div>
  </div>
  <div class="header-meta">
    {#each tags as tag}
      <span class={`meta-tag ${tag.className ?? ''}`}>{tag.label}</span>
    {/each}
  </div>
</div>

{#if tabs.length > 0}
  <div class="tabs">
    {#each tabs as tab}
      <button class="tab" class:active={activeTab === tab.key} onclick={() => onSelectTab(tab.key)}>
        {tab.label}
        {#if tab.count !== undefined && tab.count > 0}
          <span class="tab-count">{tab.count}</span>
        {/if}
      </button>
    {/each}
  </div>
{/if}

{#if body}
  {@render body()}
{/if}

<style>
  .workspace-header {
    background: rgba(61,127,255,0.04);
    border: 1px solid #1e2d4a;
    border-radius: 6px;
    margin-bottom: 1.5rem;
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .header-actions {
    display: flex;
    gap: 0.625rem;
    flex-wrap: wrap;
  }
  .header-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .btn-back {
    background: none;
    border: 1px solid #1e2d4a;
    border-radius: 4px;
    color: #8fa3cc;
    cursor: pointer;
    font-family: 'Space Mono', monospace;
    font-size: 0.7rem;
    letter-spacing: 0.05em;
    padding: 0.4rem 0.875rem;
    transition: all 0.15s;
  }
  .btn-back:hover { border-color: #3d7fff; color: #3d7fff; }
  .meta-tag {
    border-radius: 3px;
    font-family: 'Space Mono', monospace;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    padding: 0.2rem 0.5rem;
    text-transform: uppercase;
  }
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .tab-count { background: #1e2d4a; border-radius: 10px; color: #8fa3cc; font-size: 0.6rem; font-weight: 700; margin-left: 5px; padding: 1px 6px; }
</style>
