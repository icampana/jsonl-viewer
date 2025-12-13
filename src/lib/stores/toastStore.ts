import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'info';

export interface Toast {
	id: string;
	type: ToastType;
	title: string;
	message?: string;
	duration?: number;
}

export const toasts = writable<Toast[]>([]);

let toastId = 0;

export function showToast(toast: Omit<Toast, 'id'>) {
	const id = (++toastId).toString();
	const newToast: Toast = {
		id,
		duration: 5000,
		...toast
	};

	toasts.update((current) => [...current, newToast]);

	if (newToast.duration) {
		setTimeout(() => {
			removeToast(id);
		}, newToast.duration);
	}

	return id;
}

export function removeToast(id: string) {
	toasts.update((current) => current.filter((toast) => toast.id !== id));
}

export function showSuccess(title: string, message?: string) {
	return showToast({ type: 'success', title, message });
}

export function showError(title: string, message?: string) {
	return showToast({ type: 'error', title, message, duration: 8000 });
}

export function showInfo(title: string, message?: string) {
	return showToast({ type: 'info', title, message });
}