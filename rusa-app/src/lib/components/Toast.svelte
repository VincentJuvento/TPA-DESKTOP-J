<script lang="ts">
  import { toasts, dismissToast } from '$lib/stores/toast';
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div class="toast toast--{toast.type}" role="alert">
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" onclick={() => dismissToast(toast.id)}>✕</button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 1.5rem;
    right: 1.5rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 400px;
  }
  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.875rem 1.25rem;
    border-radius: 6px;
    border: 1px solid;
    font-family: 'Outfit', sans-serif;
    font-size: 0.875rem;
    animation: slideIn 0.2s ease;
    background: #0d1528;
  }
  .toast--success { border-color: #00e5a0; color: #00e5a0; }
  .toast--error { border-color: #ff4466; color: #ff4466; }
  .toast--warning { border-color: #ffaa00; color: #ffaa00; }
  .toast--info { border-color: #3d7fff; color: #3d7fff; }
  .toast-message { color: #e8eeff; }
  .toast-close {
    background: none; border: none; cursor: pointer; font-size: 0.875rem; opacity: 0.6;
    color: currentColor;
  }
  .toast-close:hover { opacity: 1; }
  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>
