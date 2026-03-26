<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { authStore, isAuthenticated, session } from '$lib/stores/auth';
  import { authApi } from '$lib/api';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import { showToast } from '$lib/stores/toast';
  import { listen } from '@tauri-apps/api/event';

  let { children } = $props();
  let initialized = $state(false);
  let unlistenBroadcast: (() => void) | null = null;

  const PUBLIC_ROUTES = ['/'];

  onMount(async () => {
    authStore.initialize();
    const s = $session;
    if (s) {
      try {
        const refreshed = await authApi.getCurrentUser(s.token);
        authStore.setSession(refreshed);
      } catch {
        authStore.clear();
      }
    }
    initialized = true;

    if (!$isAuthenticated && !PUBLIC_ROUTES.includes($page.url.pathname)) {
      goto('/');
    }

    try {
      unlistenBroadcast = await listen('new_broadcast', (event: any) => {
        const payload = event.payload;
        // Check if the current user is in the target_users array
        const currentUser = $session?.user_id;
        if (currentUser && payload.target_users && payload.target_users.includes(currentUser)) {
          showToast(`URGENT BROADCAST: ${payload.subject}`, 'error', 10000);
        }
      });
    } catch {}
  });

  onDestroy(() => {
    unlistenBroadcast?.();
  });

  let showSidebar = $derived($isAuthenticated && $page.url.pathname !== '/');
</script>

<div class="app-root">
  {#if initialized}
    {#if showSidebar}
      <Sidebar />
    {/if}
    <main class="app-main" class:full={!showSidebar}>
      {@render children()}
    </main>
  {:else}
    <div class="loading-screen">
      <div class="loading-spinner"></div>
    </div>
  {/if}
</div>

<Toast />

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    background: #05070f; color: #e8eeff;
    font-family: 'Outfit', sans-serif;
    overflow: hidden;
  }
  :global(button) { font-family: 'Outfit', sans-serif; }
  .app-root { display: flex; height: 100vh; overflow: hidden; }
  .app-main { flex: 1; overflow-y: auto; min-width: 0; }
  .app-main.full { width: 100%; }
  .loading-screen {
    display: flex; align-items: center; justify-content: center;
    width: 100%; height: 100%; background: #05070f;
  }
  .loading-spinner {
    width: 32px; height: 32px;
    border: 2px solid #1e2d4a;
    border-top-color: #3d7fff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
