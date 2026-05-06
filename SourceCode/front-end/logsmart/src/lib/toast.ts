import { toasts, type ToastType } from '$lib/stores/toast';

export function showToast(type: ToastType, title: string, details: string[] = [], duration = 5000) {
	return toasts.add(type, title, details, duration);
}

export function showSuccess(title: string, details: string[] = [], duration = 5000) {
	return showToast('success', title, details, duration);
}

export function showError(title: string, details: string[] = [], duration = 5000) {
	return showToast('error', title, details, duration);
}

export function showWarning(title: string, details: string[] = [], duration = 5000) {
	return showToast('warning', title, details, duration);
}

export function showInfo(title: string, details: string[] = [], duration = 5000) {
	return showToast('info', title, details, duration);
}
