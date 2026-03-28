<script lang="ts">
  type FilterOption = { value: string; label: string };
  type WorkspaceItem = any;

  let {
    items = [] as WorkspaceItem[],
    totalCount = 0,
    filters = [] as FilterOption[],
    selectedFilter = 'all',
    onSelectFilter = ((_: string) => {}) as (value: string) => void,
    onItemClick = ((_: WorkspaceItem) => {}) as (item: WorkspaceItem) => void,
    getTitle = ((item: WorkspaceItem) => item?.title ?? '—') as (item: WorkspaceItem) => string,
    getStatusLabel = ((item: WorkspaceItem) => item?.status ?? '—') as (item: WorkspaceItem) => string,
    getStatusClass = ((_: WorkspaceItem) => '') as (item: WorkspaceItem) => string,
    getTags = ((_: WorkspaceItem) => [] as string[]) as (item: WorkspaceItem) => string[],
    getBadges = ((_: WorkspaceItem) => [] as { label: string; className?: string }[]) as (item: WorkspaceItem) => { label: string; className?: string }[],
    getPreview = ((_: WorkspaceItem) => '') as (item: WorkspaceItem) => string,
    emptyMessage = 'No records found.',
    emptyFilteredMessage = 'No records match the selected filter.',
  } = $props();
</script>

{#if filters.length > 0}
  <div class="filter-bar">
    {#each filters as option}
      <button class="filter-btn" class:active={selectedFilter === option.value} onclick={() => onSelectFilter(option.value)}>
        {option.label}
      </button>
    {/each}
  </div>
{/if}

{#if items.length === 0}
  <p class="empty">{totalCount === 0 ? emptyMessage : emptyFilteredMessage}</p>
{:else}
  <div class="item-card-list">
    {#each items as item}
      <button class="item-card" onclick={() => onItemClick(item)}>
        <div class="item-card-header">
          <span class="item-card-title">{getTitle(item)}</span>
          <div class="item-card-badges">
            <span class="badge {getStatusClass(item)}">{getStatusLabel(item)}</span>
            {#each getBadges(item) as badge}
              <span class={`badge ${badge.className ?? ''}`}>{badge.label}</span>
            {/each}
          </div>
        </div>
        {#if getTags(item).length > 0}
          <div class="item-card-meta">
            {#each getTags(item) as tag}
              <span class="meta-chip">{tag}</span>
            {/each}
          </div>
        {/if}
        {#if getPreview(item)}
          <p class="item-card-preview">{getPreview(item)}</p>
        {/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .filter-bar { display: flex; gap: 0.375rem; flex-wrap: wrap; margin-bottom: 1rem; }
  .filter-btn { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #4a5d82; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.06em; padding: 0.2rem 0.625rem; text-transform: uppercase; transition: all 0.15s; }
  .filter-btn.active { border-color: #00d4ff; color: #00d4ff; background: rgba(0,212,255,0.08); }
  .filter-btn:hover:not(.active) { border-color: #3d7fff; color: #8fa3cc; }
  .item-card-list { display: flex; flex-direction: column; gap: 0.625rem; }
  .item-card { background: rgba(13,21,40,0.6); border: 1px solid #1e2d4a; border-radius: 6px; cursor: pointer; display: block; padding: 0.875rem 1.125rem; text-align: left; transition: border-color 0.15s, background 0.15s; width: 100%; }
  .item-card:hover { background: rgba(61,127,255,0.06); border-color: #3d7fff; }
  .item-card-header { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap; margin-bottom: 0.5rem; }
  .item-card-title { color: #e8eeff; font-size: 0.95rem; font-weight: 600; }
  .item-card-badges { display: flex; gap: 0.375rem; flex-wrap: wrap; }
  .item-card-meta { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.25rem; }
  .item-card-preview { color: #4a5d82; font-size: 0.8rem; margin: 0.4rem 0 0; line-height: 1.4; }
  .meta-chip { background: rgba(61,127,255,0.08); border-radius: 3px; color: #8fa3cc; font-size: 0.72rem; padding: 0.15rem 0.5rem; }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
</style>
