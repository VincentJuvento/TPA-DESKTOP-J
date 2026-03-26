<script lang="ts">
  import { authApi } from '$lib/api';
  import { authStore } from '$lib/stores/auth';
  import { showToast } from '$lib/stores/toast';
  import { goto } from '$app/navigation';

  let username = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state('');

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    if (!username || !password) { error = 'Please enter username and password.'; return; }
    loading = true; error = '';
    try {
      const sess = await authApi.login(username, password);
      authStore.setSession(sess);
      showToast(`Welcome, ${sess.full_name}!`, 'success');
      goto('/dashboard');
    } catch (err: any) {
      error = err?.toString() || 'Login failed. Check credentials.';
      loading = false;
    }
  }
</script>

<svelte:head><title>RUSA IMS — Login</title></svelte:head>

<div class="login-page">
  <div class="login-card">
    <div class="login-header">
      <img src="/rusa-logo.png" alt="RUSA" class="login-logo" />
      <h1 class="login-title">RUSA</h1>
      <p class="login-sub">Internal Management System</p>
      <p class="login-desc">Authorized Personnel Only</p>
    </div>

    <form class="login-form" onsubmit={handleLogin}>
      {#if error}
        <div class="login-error">{error}</div>
      {/if}
      <div class="field">
        <label class="field-label" for="login-username">Username</label>
        <input id="login-username" class="field-input" type="text" bind:value={username} placeholder="Enter username" autocomplete="username" />
      </div>
      <div class="field">
        <label class="field-label" for="login-password">Password</label>
        <input id="login-password" class="field-input" type="password" bind:value={password} placeholder="Enter password" autocomplete="current-password" />
      </div>
      <button type="submit" class="login-btn" disabled={loading}>
        {loading ? 'Authenticating...' : 'ACCESS SYSTEM'}
      </button>
    </form>

    <p class="login-footer">RUSA — Research &amp; Universal Space Agency</p>
  </div>

  <div class="login-bg">
    <div class="star" style="top:15%;left:20%;width:2px;height:2px;animation-delay:0s"></div>
    <div class="star" style="top:45%;left:65%;width:1px;height:1px;animation-delay:0.5s"></div>
    <div class="star" style="top:70%;left:35%;width:2px;height:2px;animation-delay:1s"></div>
    <div class="star" style="top:25%;left:80%;width:1px;height:1px;animation-delay:1.5s"></div>
    <div class="star" style="top:85%;left:55%;width:2px;height:2px;animation-delay:0.8s"></div>
    <div class="star" style="top:60%;left:10%;width:1px;height:1px;animation-delay:2s"></div>
  </div>
</div>

<style>
  .login-page {
    min-height: 100vh; display: flex; align-items: center; justify-content: center;
    background: #05070f; position: relative; overflow: hidden;
  }
  .login-bg { position: absolute; inset: 0; pointer-events: none; }
  .star {
    position: absolute; background: #3d7fff; border-radius: 50%;
    animation: twinkle 3s ease-in-out infinite;
  }
  @keyframes twinkle {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; box-shadow: 0 0 6px currentColor; }
  }
  .login-card {
    background: #0d1528; border: 1px solid #1e2d4a;
    border-radius: 12px; padding: 2.5rem 2.5rem 2rem;
    width: 380px; position: relative; z-index: 1;
    box-shadow: 0 0 60px rgba(61,127,255,0.1), 0 0 120px rgba(0,212,255,0.05);
  }
  .login-header { text-align: center; margin-bottom: 2rem; }
  .login-logo { width: 64px; height: 64px; object-fit: contain; margin-bottom: 1rem; }
  .login-title {
    font-family: 'Space Mono', monospace; font-size: 1.5rem;
    font-weight: 700; color: #00d4ff; letter-spacing: 0.15em; margin-bottom: 0.25rem;
  }
  .login-sub { font-size: 0.8rem; color: #8fa3cc; letter-spacing: 0.05em; margin-bottom: 0.25rem; }
  .login-desc { font-size: 0.7rem; color: #4a5d82; font-family: 'Space Mono', monospace; }
  .login-form { display: flex; flex-direction: column; gap: 1rem; }
  .login-error {
    background: rgba(255,68,102,0.1); border: 1px solid rgba(255,68,102,0.3);
    border-radius: 4px; color: #ff4466; font-size: 0.82rem; padding: 0.625rem;
  }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label {
    font-family: 'Space Mono', monospace; font-size: 0.7rem;
    text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc;
  }
  .field-input {
    background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px;
    color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem;
    padding: 0.625rem 0.875rem; transition: border-color 0.15s; width: 100%;
  }
  .field-input:focus { outline: none; border-color: #3d7fff; }
  .field-input::placeholder { color: #4a5d82; }
  .login-btn {
    background: linear-gradient(135deg, #3d7fff, #00d4ff);
    border: none; border-radius: 4px; color: #05070f;
    cursor: pointer; font-family: 'Space Mono', monospace;
    font-size: 0.8rem; font-weight: 700; letter-spacing: 0.1em;
    padding: 0.75rem; margin-top: 0.25rem; transition: opacity 0.15s;
  }
  .login-btn:hover:not(:disabled) { opacity: 0.9; }
  .login-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .login-footer { text-align: center; font-size: 0.65rem; color: #4a5d82; margin-top: 1.5rem; font-family: 'Space Mono', monospace; }
</style>
