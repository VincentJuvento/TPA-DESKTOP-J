<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    master,
    detail,
    masterWidth = '340px',
    emptyDetailMessage = 'Select an item to view details.',
    hasSelection = false,
  }: {
    master?: Snippet<[]>;
    detail?: Snippet<[]>;
    masterWidth?: string;
    emptyDetailMessage?: string;
    hasSelection?: boolean;
  } = $props();
</script>

<div class="master-detail-layout">
  <div class="master-pane" style="width: {masterWidth}; min-width: {masterWidth};">
    {#if master}
      {@render master()}
    {/if}
  </div>
  <div class="detail-pane">
    {#if hasSelection && detail}
      {@render detail()}
    {:else}
      <div class="empty-detail">
        <p class="empty-detail-msg">{emptyDetailMessage}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .master-detail-layout {
    display: flex;
    gap: 0;
    min-height: 400px;
  }
  .master-pane {
    border-right: 1px solid #1e2d4a;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding-right: 1.25rem;
    flex-shrink: 0;
  }
  .detail-pane {
    flex: 1;
    overflow-y: auto;
    padding-left: 1.5rem;
    min-width: 0;
  }
  .empty-detail {
    align-items: center;
    display: flex;
    height: 100%;
    justify-content: center;
    min-height: 200px;
  }
  .empty-detail-msg {
    color: #4a5d82;
    font-family: 'Space Mono', monospace;
    font-size: 0.75rem;
    text-align: center;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
</style>