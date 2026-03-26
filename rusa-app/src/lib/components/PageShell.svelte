<script lang="ts">
  import type { Snippet } from 'svelte';
  let {
    title = '',
    subtitle = '',
    children
  }: {
    title?: string;
    subtitle?: string;
    children?: Snippet;
  } = $props();
</script>

<div class="page-shell">
  {#if title}
    <div class="page-header">
      <div class="page-titles">
        <h1 class="page-title">{title}</h1>
        {#if subtitle}
          <p class="page-subtitle">{subtitle}</p>
        {/if}
      </div>
    </div>
  {/if}
  <div class="page-content">
    {@render children?.()}
  </div>
</div>

<style>
  .page-shell { flex: 1; display: flex; flex-direction: column; min-height: 0; }
  .page-header { padding: 1.75rem 2rem 1.25rem; border-bottom: 1px solid #1e2d4a; }
  .page-title {
    font-family: 'Space Mono', monospace;
    font-size: 1.1rem; font-weight: 700;
    color: #e8eeff; margin: 0;
    text-transform: uppercase; letter-spacing: 0.08em;
  }
  .page-subtitle { font-size: 0.85rem; color: #8fa3cc; margin: 0.25rem 0 0; }
  .page-content { flex: 1; padding: 1.5rem 2rem; overflow-y: auto; }
</style>
