import { derived } from 'svelte/store';
import type { SessionData } from '../api';
import { session } from './auth';

/**
 * Returns `true` if the user represented by `s` is authorised to perform
 * actions that require `permission`.  Authorisation is granted when:
 *   - the user's own `role_name` matches `permission`, OR
 *   - `permission` appears in the user's `inherited_permissions` list.
 *
 * Usage inside a Svelte script block:
 *   import { canPerform } from '$lib/stores/permissions';
 *   const isHead = $derived(canPerform($session, 'head_of_sanitary'));
 *
 * Usage in plain TypeScript (non-reactive):
 *   if (canPerform(session, 'disposal_crew')) { ... }
 */
export function canPerform(s: SessionData | null, permission: string): boolean {
  if (!s) return false;
  if ((s.tier ?? 0) >= 4 || s.role_name === 'the_administrator') return true;
  if (s.role_name === permission) return true;
  return (s.inherited_permissions ?? []).includes(permission);
}

/**
 * A derived store that exposes a bound `check(permission)` helper for the
 * currently authenticated session.  Useful for inline template expressions.
 *
 * Usage in a Svelte component:
 *   import { permissionChecker } from '$lib/stores/permissions';
 *   const can = $derived($permissionChecker);
 *   // then: {#if can('head_of_medicine')} ... {/if}
 */
export const permissionChecker = derived(
  session,
  ($session) => (permission: string) => canPerform($session, permission)
);
