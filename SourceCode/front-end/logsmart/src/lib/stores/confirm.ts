import { writable } from 'svelte/store';

export interface ConfirmDialogState {
	open: boolean;
	title: string;
	message: string;
	onConfirm: (() => void) | null;
	onCancel: (() => void) | null;
}

function createConfirmStore() {
	const { subscribe, set } = writable<ConfirmDialogState>({
		open: false,
		title: '',
		message: '',
		onConfirm: null,
		onCancel: null
	});

	return {
		subscribe,
		set,
		open: (title: string, message: string, onConfirm?: () => void, onCancel?: () => void) => {
			set({
				open: true,
				title,
				message,
				onConfirm: onConfirm ?? null,
				onCancel: onCancel ?? null
			});
		},
		close: () => {
			set({
				open: false,
				title: '',
				message: '',
				onConfirm: null,
				onCancel: null
			});
		}
	};
}

export const confirmDialog = createConfirmStore();
