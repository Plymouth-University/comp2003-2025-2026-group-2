import { writable, get } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
	id: number;
	type: ToastType;
	title: string;
	details: string[];
	duration: number;
}

let toastId = 0;

function createToastStore() {
	const { subscribe, update, set } = writable<Toast[]>([]);

	return {
		subscribe,
		add: (type: ToastType, title: string, details: string[] = [], duration = 5000) => {
			const id = ++toastId;
			update((toasts) => [...toasts, { id, type, title, details, duration }]);
			return id;
		},
		remove: (id: number) => {
			update((toasts) => toasts.filter((t) => t.id !== id));
		},
		clear: () => {
			set([]);
		},
		getAll: () => get({ subscribe })
	};
}

export const toasts = createToastStore();
