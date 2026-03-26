import { writable, derived } from 'svelte/store';
import type { SessionData } from '../api';

interface AuthState {
  session: SessionData | null;
  loading: boolean;
  initialized: boolean;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    session: null,
    loading: false,
    initialized: false,
  });

  return {
    subscribe,
    setSession: (session: SessionData | null) => {
      update(s => ({ ...s, session, initialized: true }));
      if (session) {
        localStorage.setItem('rusa_token', session.token);
        localStorage.setItem('rusa_session', JSON.stringify(session));
      } else {
        localStorage.removeItem('rusa_token');
        localStorage.removeItem('rusa_session');
      }
    },
    setLoading: (loading: boolean) => update(s => ({ ...s, loading })),
    initialize: () => {
      const sessionStr = localStorage.getItem('rusa_session');
      if (sessionStr) {
        try {
          const session = JSON.parse(sessionStr) as SessionData;
          set({ session, loading: false, initialized: true });
        } catch {
          set({ session: null, loading: false, initialized: true });
        }
      } else {
        update(s => ({ ...s, initialized: true }));
      }
    },
    clear: () => {
      set({ session: null, loading: false, initialized: true });
      localStorage.removeItem('rusa_token');
      localStorage.removeItem('rusa_session');
    }
  };
}

export const authStore = createAuthStore();
export const session = derived(authStore, $s => $s.session);
export const isAuthenticated = derived(authStore, $s => $s.session !== null);
