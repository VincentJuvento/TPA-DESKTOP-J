<script lang="ts">
  import { goto } from '$app/navigation';

  let {
    taskTitle,
    taskAssignedTo,
    taskStatus,
    taskType,
    allUsers = [],
  }: {
    taskTitle: string | null | undefined;
    taskAssignedTo: string | null | undefined;
    taskStatus: string | null | undefined;
    taskType: string | null | undefined;
    allUsers?: any[];
  } = $props();

  function getUserName(userId: string | null | undefined): string {
    if (!userId) return '—';
    const u = allUsers.find((u: any) => u.id === userId);
    return u ? (u.full_name ?? u.username ?? userId) : userId;
  }

  function statusClass(status: string | null | undefined): string {
    switch (status) {
      case 'pending': return 'status-badge status-pending';
      case 'in_progress': return 'status-badge status-progress';
      case 'completed': return 'status-badge status-done';
      case 'cancelled': return 'status-badge status-rejected';
      case 'conclusion_requested': return 'status-badge status-conclude';
      default: return 'status-badge';
    }
  }

  function navigateToTask() {
    if (taskType === 'aerospace') {
      goto('/aerospace');
    } else {
      goto('/life-sciences');
    }
  }
</script>

<button class="linked-task-card" onclick={navigateToTask} type="button">
  <div class="task-header">
    <span class="task-icon">🔗</span>
    <span class="task-title">{taskTitle ?? '—'}</span>
  </div>
  <div class="task-meta">
    <span class="task-meta-item">
      <span class="meta-label">Assigned to</span>
      <span class="meta-value">{getUserName(taskAssignedTo)}</span>
    </span>
    <span class="task-meta-item">
      <span class="meta-label">Status</span>
      <span class={statusClass(taskStatus)}>{taskStatus ?? '—'}</span>
    </span>
  </div>
  <span class="nav-hint">Click to view task →</span>
</button>

<style>
  .linked-task-card {
    background: rgba(61, 127, 255, 0.06);
    border: 1px solid #1e3a6e;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0.875rem 1rem;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
    width: 100%;
  }
  .linked-task-card:hover {
    background: rgba(61, 127, 255, 0.12);
    border-color: #3d7fff;
  }
  .task-header {
    align-items: center;
    display: flex;
    gap: 0.5rem;
  }
  .task-icon {
    font-size: 1rem;
  }
  .task-title {
    color: #c8d8f0;
    font-size: 0.95rem;
    font-weight: 600;
  }
  .task-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }
  .task-meta-item {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  .meta-label {
    color: #4a5d82;
    font-family: 'Space Mono', monospace;
    font-size: 0.6rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .meta-value {
    color: #8fa3cc;
    font-size: 0.85rem;
  }
  .status-badge {
    border-radius: 3px;
    font-family: 'Space Mono', monospace;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
    padding: 0.15rem 0.45rem;
    text-transform: uppercase;
  }
  .status-pending { background: rgba(245, 158, 11, 0.15); color: #f59e0b; }
  .status-progress { background: rgba(61, 127, 255, 0.15); color: #3d7fff; }
  .status-done { background: rgba(16, 185, 129, 0.15); color: #10b981; }
  .status-rejected { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
  .status-conclude { background: rgba(139, 92, 246, 0.15); color: #8b5cf6; }
  .nav-hint {
    color: #3d7fff;
    font-family: 'Space Mono', monospace;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
  }
</style>
