<script lang="ts">
  import type { Snippet } from 'svelte';
  let {
    open = $bindable(false),
    title = '',
    onclose = () => {},
    children
  }: {
    open?: boolean;
    title?: string;
    onclose?: () => void;
    children?: Snippet;
  } = $props();

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
      onclose();
    }
  }
</script>

{#if open}
  <div
    class="modal-backdrop"
    onclick={handleBackdrop}
    onkeydown={(e) => e.key === 'Escape' && (open = false, onclose())}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="modal-box">
      <div class="modal-header">
        <span class="modal-title">{title}</span>
        <button class="modal-close" onclick={() => { open = false; onclose(); }}>✕</button>
      </div>
      <div class="modal-body">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed; inset: 0; background: rgba(5,7,15,0.85);
    display: flex; align-items: center; justify-content: center;
    z-index: 1000; backdrop-filter: blur(4px);
  }
  .modal-box {
    background: #0d1528; border: 1px solid #1e2d4a;
    border-radius: 8px; min-width: 500px; max-width: 700px;
    width: 90%; max-height: 85vh; overflow-y: auto;
    box-shadow: 0 0 40px rgba(61,127,255,0.15);
  }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 1.25rem 1.5rem; border-bottom: 1px solid #1e2d4a;
  }
  .modal-title {
    font-family: 'Space Mono', monospace; font-size: 0.85rem;
    text-transform: uppercase; letter-spacing: 0.08em; color: #00d4ff;
  }
  .modal-close {
    background: none; border: none; cursor: pointer; color: #4a5d82;
    font-size: 1rem; transition: color 0.15s;
  }
  .modal-close:hover { color: #e8eeff; }
  .modal-body { padding: 1.5rem; }
</style>
