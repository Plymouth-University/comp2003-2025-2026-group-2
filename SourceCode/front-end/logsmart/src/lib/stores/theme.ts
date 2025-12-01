import { writable } from 'svelte/store';
import { browser } from '$app/environment';

function createThemeStore() {
	const { subscribe, set, update } = writable<boolean>(false);

	return {
		subscribe,
		initialize: () => {
			if (browser) {
				const saved = localStorage.getItem('theme');
				const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
				const isDark = saved === 'dark' || (!saved && prefersDark);
				set(isDark);
				if (isDark) {
					document.documentElement.classList.add('dark');
				} else {
					document.documentElement.classList.remove('dark');
				}
			}
		},
		toggle: () => {
			if (browser) {
				update((isDark) => {
					const newValue = !isDark;
					if (newValue) {
						document.documentElement.classList.add('dark');
						localStorage.setItem('theme', 'dark');
					} else {
						document.documentElement.classList.remove('dark');
						localStorage.setItem('theme', 'light');
					}
					return newValue;
				});
			}
		},
		set: (value: boolean) => {
			if (browser) {
				set(value);
				if (value) {
					document.documentElement.classList.add('dark');
					localStorage.setItem('theme', 'dark');
				} else {
					document.documentElement.classList.remove('dark');
					localStorage.setItem('theme', 'light');
				}
			}
		}
	};
}

export const isDarkMode = createThemeStore();
