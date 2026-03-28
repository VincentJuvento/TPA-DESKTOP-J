<script lang="ts">
  import { session } from '$lib/stores/auth';
  import { authStore } from '$lib/stores/auth';
  import { authApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { canPerform } from '$lib/stores/permissions';

  let currentPath = $derived($page.url.pathname);

  interface NavItem {
    path: string;
    label: string;
    icon: string;
    roles?: string[];
    minTier?: number;
  }

  const navItems: NavItem[] = [
    { path: '/dashboard', label: 'Dashboard', icon: '⌂' },
    { path: '/messages', label: 'Messages', icon: '✉' },
    { path: '/research', label: 'Research & Lab', icon: '⚗', roles: ['biologist', 'biological_engineer', 'agricultural_engineer', 'the_observer', 'the_taskmaster'] },
    { path: '/chemistry', label: 'Chemistry', icon: '🧪', roles: ['chemist', 'physicist', 'the_observer'] },
    { path: '/theoretical', label: 'Theoretical Sciences', icon: '∑', roles: ['mathematician', 'the_artificer', 'the_taskmaster'] },
    { path: '/data-services', label: 'Data Services', icon: '◈' },
    { path: '/data-request', label: 'Request Data', icon: '◇' },
    { path: '/security', label: 'Security', icon: '⚔', roles: ['earth_security_head', 'earth_security_staff', 'galactic_security_head', 'galactic_security_staff', 'the_guardian', 'the_overseer'] },
    { path: '/communications', label: 'Communications', icon: '📡', roles: ['the_anchorman', 'the_guardian', 'the_overseer'] },
    { path: '/astronautics', label: 'Astronautics', icon: '🚀', roles: ['astronaut', 'the_wanderer', 'the_taskmaster'] },
    { path: '/aerospace', label: 'Aerospace Engineering', icon: '🛠', roles: ['aerospace_engineer'] },
    { path: '/settlement', label: 'Settlement Ops', icon: '🏗', roles: ['settler_commander', 'civil_engineer', 'farmer'] },
    { path: '/space-station', label: 'Space Station', icon: '🛸', roles: ['space_station_settler'] },
    { path: '/psychiatry', label: 'Psychiatry', icon: '🧠', roles: ['psychiatrist', 'psychiatrist_assistant'] },
    { path: '/medical', label: 'Medical Services', icon: '⚕', roles: ['head_of_medicine', 'medical_staff'] },
    { path: '/sanitary', label: 'Sanitary & Waste', icon: '♺', roles: ['head_of_sanitary', 'cleanup_crew', 'disposal_crew', 'wastewater_crew', 'transport_crew', 'sanitary_inspector'] },
    { path: '/budget', label: 'Budget & Finance', icon: '₿', roles: ['the_accountant', 'head_of_medicine', 'head_of_sanitary', 'earth_security_head', 'galactic_security_head'] },
    { path: '/governance', label: 'Governance', icon: '⚖', minTier: 3 },
    { path: '/admin', label: 'Administration', icon: '⚙', roles: ['the_administrator'] },
  ];

  function canSee(item: NavItem): boolean {
    const s = $session;
    if (!s) return false;
    if (item.minTier && s.tier < item.minTier) return false;
    if (item.roles) {
      // Allow if own role or an inherited role matches any entry in the list
      const hasRole = item.roles.some(r => canPerform(s, r));
      if (!hasRole && s.tier < 4) return false;
    }
    return true;
  }

  async function handleLogout() {
    const s = $session;
    if (s) {
      try { await authApi.logout(s.token); } catch {}
    }
    authStore.clear();
    goto('/');
    showToast('Logged out successfully', 'info');
  }
</script>

<aside class="sidebar">
  <div class="sidebar-logo">
    <img src="/rusa-logo.png" alt="RUSA" class="logo-img" />
    <div class="logo-text">
      <span class="logo-name">RUSA</span>
      <span class="logo-sub">Internal Management System</span>
    </div>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      {#if canSee(item)}
        <a href={item.path} class="nav-item" class:active={currentPath.startsWith(item.path)}>
          <span class="nav-icon">{item.icon}</span>
          <span class="nav-label">{item.label}</span>
        </a>
      {/if}
    {/each}
  </nav>

  <div class="sidebar-user">
    {#if $session}
      <div class="user-info">
        <span class="user-name">{$session.full_name}</span>
        <span class="user-role">{$session.role_display_name}</span>
      </div>
      <button class="logout-btn" onclick={handleLogout}>Logout</button>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 240px; min-width: 240px;
    background: #080d1a;
    border-right: 1px solid #1e2d4a;
    display: flex; flex-direction: column;
    height: 100vh; overflow-y: auto;
    position: sticky; top: 0;
  }
  .sidebar-logo {
    display: flex; align-items: center; gap: 0.75rem;
    padding: 1.25rem 1.25rem 1rem;
    border-bottom: 1px solid #1e2d4a;
  }
  .logo-img { width: 36px; height: 36px; object-fit: contain; }
  .logo-text { display: flex; flex-direction: column; }
  .logo-name {
    font-family: 'Space Mono', monospace; font-size: 0.9rem;
    font-weight: 700; color: #00d4ff; letter-spacing: 0.1em;
  }
  .logo-sub { font-size: 0.6rem; color: #4a5d82; letter-spacing: 0.05em; }
  .sidebar-nav { flex: 1; padding: 0.75rem 0; overflow-y: auto; }
  .nav-item {
    display: flex; align-items: center; gap: 0.75rem;
    padding: 0.6rem 1.25rem;
    color: #8fa3cc; text-decoration: none;
    font-family: 'Outfit', sans-serif; font-size: 0.85rem;
    transition: all 0.15s; border-left: 2px solid transparent;
  }
  .nav-item:hover { color: #e8eeff; background: rgba(61,127,255,0.06); }
  .nav-item.active { color: #00d4ff; background: rgba(0,212,255,0.07); border-left-color: #00d4ff; }
  .nav-icon { font-size: 0.9rem; width: 18px; text-align: center; }
  .nav-label { font-size: 0.82rem; }
  .sidebar-user { padding: 1rem 1.25rem; border-top: 1px solid #1e2d4a; display: flex; flex-direction: column; gap: 0.5rem; }
  .user-info { display: flex; flex-direction: column; gap: 0.1rem; }
  .user-name { font-family: 'Outfit', sans-serif; font-size: 0.85rem; color: #e8eeff; font-weight: 500; }
  .user-role { font-family: 'Space Mono', monospace; font-size: 0.65rem; color: #4a5d82; }
  .logout-btn {
    background: none; border: 1px solid #1e2d4a; border-radius: 4px;
    color: #8fa3cc; cursor: pointer; font-family: 'Outfit', sans-serif;
    font-size: 0.8rem; padding: 0.4rem 0.75rem; transition: all 0.15s; text-align: left;
  }
  .logout-btn:hover { border-color: #ff4466; color: #ff4466; }
</style>
