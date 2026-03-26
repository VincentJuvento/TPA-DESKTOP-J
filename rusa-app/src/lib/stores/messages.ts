import { writable } from 'svelte/store';

interface MessagesState {
  inbox: any[];
  sent: any[];
  unreadCount: number;
}

export const messagesStore = writable<MessagesState>({
  inbox: [],
  sent: [],
  unreadCount: 0,
});

export function setInbox(messages: any[]) {
  messagesStore.update(s => ({
    ...s,
    inbox: messages,
    unreadCount: messages.filter((m: any) => !m.read_at).length
  }));
}
