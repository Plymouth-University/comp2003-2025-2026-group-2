import { confirmDialog } from '$lib/stores/confirm';

export function confirm(title: string, message: string, onConfirm?: () => void, onCancel?: () => void) {
	confirmDialog.open(title, message, onConfirm, onCancel);
}