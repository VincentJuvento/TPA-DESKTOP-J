import { writable } from 'svelte/store';

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration?: number;
}

const { subscribe, update } = writable<Toast[]>([]);

export const toasts = { subscribe };

export function showToast(message: string, type: Toast['type'] = 'info', duration = 4000) {
  const id = crypto.randomUUID();
  update(ts => [...ts, { id, type, message, duration }]);
  if (duration > 0) {
    setTimeout(() => {
      update(ts => ts.filter(t => t.id !== id));
    }, duration);
  }
  return id;
}

export function dismissToast(id: string) {
  update(ts => ts.filter(t => t.id !== id));
}
