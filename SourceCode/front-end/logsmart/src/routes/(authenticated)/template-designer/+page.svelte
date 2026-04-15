<script lang="ts">
	import { browser } from '$app/environment';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import { onMount } from 'svelte';
	import TemplatesSidebar from './TemplatesSidebar.svelte';
	import DesignCanvas from './DesignCanvas.svelte';
	import ComponentsPalette from './ComponentsPalette.svelte';
	import PropertiesPanel from './PropertiesPanel.svelte';
	import AiGeneratorSidebar from './AiGeneratorSidebar.svelte';
	import VersionHistoryModal from './VersionHistoryModal.svelte';
	import { DEFAULT_TEMPLATE_BLUEPRINTS } from './defaultTemplates';
	import type { CanvasItem, ComponentType, Template } from './types';
	import type { PageData } from './$types';
	import type { components } from '$lib/api-types';
	let { data } = $props<{ data: PageData }>();
	let templates = $state<Template[]>([]);

	const componentTypes: ComponentType[] = [
		{ type: 'text_input', name: 'Text Input', icon: 'T' },
		{ type: 'checkbox', name: 'Checkbox', icon: '✓' },
		{ type: 'temperature', name: 'Temperature Input', icon: '🌡' },
		{ type: 'dropdown', name: 'Dropdown', icon: '≡' },
		{ type: 'label', name: 'Text Label', icon: 'L' }
	];

	let canvasItems = $state<CanvasItem[]>([]);
	let logTitle = $state('');
	let selectedItemId = $state<string | null>(null);
	let isEditing = $state(false);
	let originalTemplateName = $state<string | null>(null);
	let loading = $state(false);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let saveSuccess = $state(false);
	let versionName = $state('');
	const DEFAULT_NEW_TEMPLATE_SCHEDULE: components['schemas']['Schedule'] = {
		frequency: 'Daily',
		days_of_week: [1, 2, 3, 4, 5]
	};
	let newTemplateSchedule = $state<components['schemas']['Schedule']>({
		...DEFAULT_NEW_TEMPLATE_SCHEDULE
	});
	let deleting = $state(false);
	let deleteError = $state<string | null>(null);
	let hasUnsavedChanges = $state(false);
	let aiPrompt = $state('');
	let aiLoading = $state(false);
	let aiError = $state<string | null>(null);
	let canvasItemsBackup = $state<CanvasItem[] | null>(null);
	let hasUndoAvailable = $derived(canvasItemsBackup !== null);

	let showHistory = $state(false);
	let historyVersions = $state<components['schemas']['TemplateVersionInfo'][]>([]);
	// restore state is derived from async action; no UI binding needed
	let currentVersion = $state<number>(1);
	let currentVersionName = $state<string | null>(null);
	let branches = $state<{ id: string; name: string }[]>([]);
	const canManageCompany = $derived(
		data.user?.role === 'company_manager' ||
			data.user?.role === 'branch_manager' ||
			data.user?.role === 'logsmart_admin'
	);
	let branchId = $derived<string>(data.user?.branch_id || '');
	$effect(() => {
		if (canManageCompany && branchId === '') {
			branchId = 'company';
		}
	});

	const templateId = $derived(page.url.searchParams.get('id'));

	function mapApiFieldToCanvasItem(
		field: components['schemas']['TemplateField'],
		index: number
	): CanvasItem {
		return {
			id: `item_${index}_${Math.random().toString(36).substring(2, 9)}`,
			type: field.field_type,
			x: field.position.x,
			y: field.position.y,
			props: {
				text: field.props.text ?? '',
				placeholder: field.props.placeholder ?? '',
				size: field.props.size ? parseInt(field.props.size) : 16,
				weight: field.props.weight ?? 'normal',
				editable: field.props.editable ?? true,
				min: field.props.min ?? 0,
				max: field.props.max ?? 100,
				value: field.props.value ?? '',
				unit: field.props.unit ?? '°C',
				options: field.props.options ?? [],
				selected: field.props.selected ?? false,
				fontFamily: field.props.font_family ?? 'system-ui',
				textDecoration: field.props.text_decoration ?? 'none',
				color: field.props.color ?? '',
				required: field.props.required ?? false,
				maxLength: field.props.max_length ?? null,
				minLength: field.props.min_length ?? null,
				inputType: field.props.input_type ?? 'text'
			}
		};
	}

	function mapCanvasItemToApiField(item: CanvasItem): components['schemas']['TemplateField'] {
		const props: components['schemas']['TemplateFieldProps'] = {};

		if (item.props.text !== undefined) props.text = item.props.text;
		if (item.props.placeholder !== undefined) props.placeholder = item.props.placeholder;
		if (item.props.size !== undefined) props.size = String(item.props.size);
		if (item.props.weight !== undefined) props.weight = item.props.weight;
		if (item.props.editable !== undefined) props.editable = item.props.editable;
		if (item.props.min !== undefined) props.min = item.props.min;
		if (item.props.max !== undefined) props.max = item.props.max;
		if (item.props.value !== undefined) props.value = String(item.props.value);
		if (item.props.unit !== undefined) props.unit = item.props.unit;
		if (item.props.options !== undefined) props.options = item.props.options;
		if (item.props.selected !== undefined) props.selected = String(item.props.selected);
		if (item.props.fontFamily !== undefined) props.font_family = item.props.fontFamily;
		if (item.props.textDecoration !== undefined) props.text_decoration = item.props.textDecoration;
		if (item.props.color !== undefined) props.color = item.props.color;
		if (item.props.required !== undefined) props.required = item.props.required;
		if (item.props.maxLength !== undefined) props.max_length = item.props.maxLength;
		if (item.props.minLength !== undefined) props.min_length = item.props.minLength;
		if (item.props.inputType !== undefined) props.input_type = item.props.inputType;

		return {
			field_type: item.type,
			position: { x: item.x, y: item.y },
			props
		};
	}

	async function loadTemplate(name: string) {
		loading = true;
		saveError = null;

		const { data, error } = await api.GET('/logs/templates', {
			params: {
				query: {
					template_name: name
				}
			}
		});

		if (error) {
			console.error('Failed to load template:', error);
			loading = false;
			return;
		}

		if (data) {
			logTitle = data.template_name;
			originalTemplateName = data.template_name;
			canvasItems = data.template_layout.map(mapApiFieldToCanvasItem);
			isEditing = true;
			currentVersion = data.version || 1;
			currentVersionName = data.version_name || null;
			branchId = data.branch_id || 'company';
		}
		loading = false;
		hasUnsavedChanges = false;
	}

	async function saveTemplate() {
		if (!logTitle.trim()) {
			saveError = 'Template name is required';
			return;
		}

		saving = true;
		saveError = null;

		const templateLayout = canvasItems.map(mapCanvasItemToApiField);

		if (isEditing && originalTemplateName) {
			if (logTitle !== originalTemplateName) {
				const { error: renameError } = await api.PUT('/logs/templates/rename', {
					body: {
						old_template_name: originalTemplateName,
						new_template_name: logTitle
					}
				});

				if (renameError) {
					saveError = 'Failed to rename template';
					saving = false;
					return;
				}

				originalTemplateName = logTitle;
			}

			const { error } = await api.PUT('/logs/templates/update', {
				body: {
					template_name: logTitle,
					template_layout: templateLayout,
					version_name: versionName || null,
					branch_id: branchId === 'company' ? 'company' : branchId
				}
			});

			if (error) {
				saveError = 'Failed to update template';
				saving = false;
				return;
			}
		} else {
			const { error } = await api.POST('/logs/templates', {
				body: {
					template_name: logTitle,
					template_layout: templateLayout,
					schedule: newTemplateSchedule,
					branch_id: branchId === 'company' || branchId === '' ? undefined : branchId
				}
			});

			if (error) {
				saveError = 'Failed to save template';
				saving = false;
				return;
			}

			isEditing = true;
			originalTemplateName = logTitle;
		}

		saveSuccess = true;
		saving = false;
		hasUnsavedChanges = false;
		versionName = '';

		// Reload the template to get updated version info
		if (isEditing && originalTemplateName) {
			await loadTemplate(originalTemplateName);
		}

		// Refresh version history if the modal is open
		if (showHistory && originalTemplateName) {
			const { data } = await api.GET('/logs/templates/versions', {
				params: {
					query: {
						template_name: originalTemplateName
					}
				}
			});
			if (data?.versions) {
				historyVersions = data.versions;
			}
		}

		await fetchTemplates();

		setTimeout(() => {
			saveSuccess = false;
		}, 3000);
	}

	async function deleteTemplate() {
		if (!isEditing || !originalTemplateName) {
			deleteError = 'No template to delete';
			return;
		}

		if (
			!confirm(
				`Are you sure you want to delete "${originalTemplateName}"? This action cannot be undone.`
			)
		) {
			return;
		}

		deleting = true;
		deleteError = null;

		const { error } = await api.DELETE('/logs/templates', {
			params: {
				query: {
					template_name: originalTemplateName
				}
			}
		});

		if (error) {
			deleteError = 'Failed to delete template';
			deleting = false;
			return;
		}

		deleting = false;
		createNewTemplate();
		await fetchTemplates();
	}

	onMount(() => {
		if (browser && templateId) {
			loadTemplate(templateId);
		}
	});

	let previousTitle = $state('');
	$effect(() => {
		if (logTitle !== previousTitle && previousTitle !== '') {
			hasUnsavedChanges = true;
		}
		previousTitle = logTitle;
	});

	async function fetchTemplates() {
		const { data } = await api.GET('/logs/templates/all');
		if (data?.templates) {
			templates = data.templates.map((t: components['schemas']['TemplateInfo'], index: number) => ({
				id: index,
				name: t.template_name,
				selected: t.template_name === originalTemplateName
			}));
		}

		const { data: branchesData } = await api.GET('/auth/company/branches');
		if (branchesData?.branches) {
			branches = branchesData.branches;
		}
	}

	onMount(() => {
		fetchTemplates();
	});

	function createNewTemplate() {
		if (hasUnsavedChanges) {
			if (!confirm('You have unsaved changes. Are you sure you want to create a new template?')) {
				return;
			}
		}
		canvasItems = [];
		logTitle = '';
		selectedItemId = null;
		isEditing = false;
		originalTemplateName = null;
		newTemplateSchedule = { ...DEFAULT_NEW_TEMPLATE_SCHEDULE };
		branchId = data.user?.branch_id || (canManageCompany ? 'company' : '');
		hasUnsavedChanges = false;
		goto('/template-designer', { replaceState: true });
	}

	function useDefaultTemplate(templateId: string) {
		const blueprint = DEFAULT_TEMPLATE_BLUEPRINTS.find((t) => t.id === templateId);
		if (!blueprint) return;

		if (hasUnsavedChanges) {
			if (!confirm('You have unsaved changes. Are you sure you want to load a default template?')) {
				return;
			}
		}

		canvasItems = blueprint.template_layout.map(mapApiFieldToCanvasItem);
		logTitle = `${blueprint.name} Copy`;
		selectedItemId = null;
		isEditing = false;
		originalTemplateName = null;
		newTemplateSchedule = { ...blueprint.schedule };
		branchId = data.user?.branch_id || (canManageCompany ? 'company' : '');
		hasUnsavedChanges = true;
		goto('/template-designer', { replaceState: true });
	}

	function selectTemplate(templateName: string) {
		if (templateName === originalTemplateName) return;
		if (hasUnsavedChanges) {
			if (!confirm('You have unsaved changes. Are you sure you want to switch templates?')) {
				return;
			}
		}
		goto(`/template-designer?id=${encodeURIComponent(templateName)}`, { replaceState: true });
		loadTemplate(templateName);
	}

	function generateId() {
		return Math.random().toString(36).substring(2, 9);
	}

	function getDefaultProps(type: string): Record<string, unknown> {
		switch (type) {
			case 'text_input':
				return {
					text: '',
					size: 16,
					weight: 'normal',
					placeholder: 'Text Input',
					fontFamily: 'system-ui',
					textDecoration: 'none',
					color: '',
					required: false,
					maxLength: null,
					minLength: null,
					inputType: 'text'
				};
			case 'checkbox':
				return {
					text: 'Checkbox Label',
					size: '16px',
					weight: 'normal',
					color: '',
					required: false
				};
			case 'temperature':
				return { value: 0, min: -20, max: 50, label: 'Temperature', unit: '°C' };
			case 'dropdown':
				return { selected: '', options: ['Option 1', 'Option 2', 'Option 3'] };
			case 'label':
				return {
					editable: true,
					text: 'Label Text',
					size: 16,
					weight: 'normal',
					fontFamily: 'system-ui',
					textDecoration: 'none',
					color: ''
				};
			default:
				return {};
		}
	}

	function addComponent(type: string, x: number, y: number) {
		const newItem: CanvasItem = {
			id: generateId(),
			type,
			x,
			y,
			props: getDefaultProps(type)
		};
		canvasItems.push(newItem);
		selectedItemId = newItem.id;
		hasUnsavedChanges = true;
	}

	function deleteSelected() {
		if (selectedItemId) {
			canvasItems = canvasItems.filter((item) => item.id !== selectedItemId);
			selectedItemId = null;
			hasUnsavedChanges = true;
		}
	}

	function constrainToBounds(x: number, y: number, itemId: string): { x: number; y: number } {
		if (!canvasRef) return { x, y };

		const canvasRect = canvasRef.getBoundingClientRect();
		const itemElement = canvasRef.querySelector(`[data-item-id="${itemId}"]`) as HTMLElement;
		if (!itemElement) return { x, y };

		const itemRect = itemElement.getBoundingClientRect();
		const itemWidth = itemRect.width;
		const itemHeight = itemRect.height;

		// Ensure item stays within canvas bounds
		const constrainedX = Math.max(0, Math.min(x, canvasRect.width - itemWidth));
		const constrainedY = Math.max(0, Math.min(y, canvasRect.height - itemHeight));

		return { x: constrainedX, y: constrainedY };
	}

	function updateItemProp(itemId: string, propKey: string, value: unknown) {
		if (propKey === 'lockX' || propKey === 'lockY' || propKey === 'x' || propKey === 'y') {
			// Constrain x and y positions to canvas bounds
			if (propKey === 'x' || propKey === 'y') {
				const item = canvasItems.find((i) => i.id === itemId);
				if (item) {
					const newX = propKey === 'x' ? (value as number) : item.x;
					const newY = propKey === 'y' ? (value as number) : item.y;
					const constrained = constrainToBounds(newX, newY, itemId);
					canvasItems = canvasItems.map((i) =>
						i.id === itemId ? { ...i, x: constrained.x, y: constrained.y } : i
					);
				}
			} else {
				canvasItems = canvasItems.map((item) =>
					item.id === itemId ? { ...item, [propKey]: value } : item
				);
			}
		} else {
			canvasItems = canvasItems.map((item) =>
				item.id === itemId ? { ...item, props: { ...item.props, [propKey]: value } } : item
			);
		}
		hasUnsavedChanges = true;
	}

	let selectedItem = $derived(canvasItems.find((item) => item.id === selectedItemId));

	let canvasRef = $state<HTMLDivElement | null>(null);

	function alignItem(
		itemId: string,
		horizontal: 'left' | 'center' | 'right' | null,
		vertical: 'top' | 'center' | 'bottom' | null
	) {
		if (!canvasRef) return;

		const canvasRect = canvasRef.getBoundingClientRect();
		const itemElement = canvasRef.querySelector(`[data-item-id="${itemId}"]`) as HTMLElement;
		if (!itemElement) return;

		const itemRect = itemElement.getBoundingClientRect();
		const itemWidth = itemRect.width;
		const itemHeight = itemRect.height;

		canvasItems = canvasItems.map((item) => {
			if (item.id !== itemId) return item;

			let newX = item.x;
			let newY = item.y;

			if (horizontal === 'left') {
				newX = 0;
			} else if (horizontal === 'center') {
				newX = (canvasRect.width - itemWidth) / 2;
			} else if (horizontal === 'right') {
				newX = canvasRect.width - itemWidth;
			}

			if (vertical === 'top') {
				newY = 0;
			} else if (vertical === 'center') {
				newY = (canvasRect.height - itemHeight) / 2;
			} else if (vertical === 'bottom') {
				newY = canvasRect.height - itemHeight;
			}

			return { ...item, x: newX, y: newY };
		});
		hasUnsavedChanges = true;
	}

	function markUnsavedChanges() {
		hasUnsavedChanges = true;
	}

	async function generateLayoutFromPrompt() {
		if (!aiPrompt.trim()) {
			aiError = 'Please enter a prompt';
			setTimeout(() => {
				aiError = null;
			}, 3000);
			return;
		}

		aiLoading = true;
		aiError = null;

		try {
			let promptWithContext = '';

			if (canvasRef) {
				const rect = canvasRef.getBoundingClientRect();
				const canvasWidth = Math.round(rect.width);
				const canvasHeight = Math.round(rect.height);
				promptWithContext += ` Canvas dimensions: ${canvasWidth}px width × ${canvasHeight}px height.`;

				const componentDimensions: Record<string, { width: number; height: number }> = {
					text_input: { width: 260, height: 50 },
					checkbox: { width: 100, height: 60 },
					temperature: { width: 440, height: 80 },
					dropdown: { width: 160, height: 60 },
					label: { width: 100, height: 40 }
				};

				const componentSizes = componentTypes
					.map((comp) => {
						const dims = componentDimensions[comp.type] || { width: 150, height: 40 };
						return `${comp.name} (${comp.type}): ${dims.width}px × ${dims.height}px`;
					})
					.join(', ');

				promptWithContext += ` Available components with typical sizes: ${componentSizes}.`;
				promptWithContext += ` User prompt: \n ${aiPrompt}`;
			}

			const { data, error } = await api.POST('/llm/generate-layout', {
				body: {
					user_prompt: promptWithContext
				}
			});

			if (error) {
				aiError = 'Failed to generate layout';
				setTimeout(() => {
					aiError = null;
				}, 3000);
				aiLoading = false;
				return;
			}

			if (!data || typeof data !== 'object') {
				aiError = 'Invalid response from backend';
				setTimeout(() => {
					aiError = null;
				}, 3000);
				aiLoading = false;
				return;
			}

			const layoutObj = data as { layout?: { template_layout?: unknown } };
			const layoutData = layoutObj.layout?.template_layout;

			if (!Array.isArray(layoutData)) {
				aiError = 'Invalid response from backend';
				setTimeout(() => {
					aiError = null;
				}, 3000);
				aiLoading = false;
				return;
			}

			canvasItemsBackup = [...canvasItems];
			canvasItems = layoutData.map(mapApiFieldToCanvasItem);
			selectedItemId = null;
			hasUnsavedChanges = true;
			aiPrompt = '';
			aiLoading = false;
		} catch (e) {
			aiError = `Error: ${e instanceof Error ? e.message : 'Unknown error'}`;
			setTimeout(() => {
				aiError = null;
			}, 3000);
			aiLoading = false;
		}
	}

	function undoGeneration() {
		if (canvasItemsBackup !== null) {
			canvasItems = canvasItemsBackup;
			canvasItemsBackup = null;
			selectedItemId = null;
		}
	}

	let leftPaletteHeight = $state<number | null>(null);
	let rightPaletteHeight = $state<number | null>(null);
	let resizingSidebar = $state<'left' | 'right' | null>(null);

	// Responsive sidebar state
	let leftSidebarOpen = $state(true);
	let rightSidebarOpen = $state(true);
	let isSmallScreen = $state(false);

	// Check screen size and update sidebar visibility
	$effect(() => {
		if (!browser) return;

		function handleResize() {
			const width = window.innerWidth;
			isSmallScreen = width < 1280; // xl breakpoint

			// Auto-close sidebars on small screens initially
			if (isSmallScreen && leftSidebarOpen && rightSidebarOpen) {
				leftSidebarOpen = false;
				rightSidebarOpen = false;
			}
		}

		handleResize();
		window.addEventListener('resize', handleResize);

		return () => {
			window.removeEventListener('resize', handleResize);
		};
	});

	function toggleLeftSidebar() {
		leftSidebarOpen = !leftSidebarOpen;
	}

	function toggleRightSidebar() {
		rightSidebarOpen = !rightSidebarOpen;
	}

	function closeDrawers() {
		if (isSmallScreen) {
			leftSidebarOpen = false;
			rightSidebarOpen = false;
		}
	}

	function handleResizeStart(e: MouseEvent, side: 'left' | 'right') {
		e.preventDefault();
		resizingSidebar = side;
		window.addEventListener('mousemove', handleResizeMove);
		window.addEventListener('mouseup', handleResizeEnd);
	}

	function handleResizeMove(e: MouseEvent) {
		if (!resizingSidebar) return;

		const sidebarSelector =
			resizingSidebar === 'left' ? '[data-left-sidebar]' : '[data-right-sidebar]';
		const sidebar = document.querySelector(sidebarSelector);

		if (sidebar) {
			const sidebarRect = sidebar.getBoundingClientRect();
			const currentHeight = resizingSidebar === 'left' ? leftPaletteHeight : rightPaletteHeight;

			if (currentHeight === null) {
				const initialHeight = sidebarRect.height / 2;
				if (resizingSidebar === 'left') {
					leftPaletteHeight = initialHeight;
				} else {
					rightPaletteHeight = initialHeight;
				}
			}

			const newHeight = e.clientY - sidebarRect.top;
			const constrainedHeight = Math.max(100, Math.min(newHeight, sidebarRect.height - 100));

			if (resizingSidebar === 'left') {
				leftPaletteHeight = constrainedHeight;
			} else {
				rightPaletteHeight = constrainedHeight;
			}
		}
	}

	function handleResizeEnd() {
		resizingSidebar = null;
		window.removeEventListener('mousemove', handleResizeMove);
		window.removeEventListener('mouseup', handleResizeEnd);
	}

	async function fetchVersions() {
		if (!originalTemplateName) return;

		const { data, error } = await api.GET('/logs/templates/versions', {
			params: {
				query: {
					template_name: originalTemplateName
				}
			}
		});

		if (error) {
			console.error('Failed to fetch versions:', error);
			alert('Failed to load version history');
			return;
		}

		if (data?.versions) {
			historyVersions = data.versions;
			showHistory = true;
		}
	}

	async function restoreVersion(version: number) {
		if (!originalTemplateName) return;
		if (
			!confirm(
				`Are you sure you want to restore version ${version}? Current changes will be saved as a new version.`
			)
		)
			return;

		const restoring = true;
		const { error } = await api.POST('/logs/templates/versions/restore', {
			params: {
				query: {
					template_name: originalTemplateName
				}
			},
			body: {
				version
			}
		});

		if (error) {
			console.error('Failed to restore version:', error);
			alert('Failed to restore version');
			void restoring;
			return;
		}

		// Reload the template to reflect changes
		await loadTemplate(originalTemplateName);
		showHistory = false;
		void restoring;
		saveSuccess = true;
		setTimeout(() => {
			saveSuccess = false;
		}, 3000);
	}
</script>

<svelte:head>
	<title>Templates Dashboard</title>
</svelte:head>

<VersionHistoryModal
	isOpen={showHistory}
	versions={historyVersions}
	{currentVersion}
	{currentVersionName}
	onClose={() => (showHistory = false)}
	onRestore={restoreVersion}
/>

<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<!-- Centered menu bar for small screens -->
	{#if isSmallScreen}
		<div class="fixed bottom-4 left-1/2 z-50 -translate-x-1/2 transform">
			<div class="menu-bar flex items-center gap-1 rounded-full px-4 py-2 shadow-xl">
				<button
					class="menu-btn rounded-full px-4 py-2"
					class:menu-btn-active={leftSidebarOpen}
					onclick={toggleLeftSidebar}
					aria-label="Toggle templates/AI sidebar"
					title="Templates & AI"
				>
					<span class="text-lg">{leftSidebarOpen ? '✕' : '☰'}</span>
					<span class="ml-1.5 text-sm">Templates</span>
				</button>
				<div class="menu-divider"></div>
				<button
					class="menu-btn rounded-full px-4 py-2"
					class:menu-btn-active={rightSidebarOpen}
					onclick={toggleRightSidebar}
					aria-label="Toggle components/properties sidebar"
					title="Components & Properties"
				>
					<span class="text-lg">{rightSidebarOpen ? '✕' : '🎨'}</span>
					<span class="ml-1.5 text-sm">Components</span>
				</button>
			</div>
		</div>
	{/if}

	<!-- Overlay for drawer mode -->
	{#if isSmallScreen && (leftSidebarOpen || rightSidebarOpen)}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="fixed inset-0 z-30 bg-black/50" onclick={closeDrawers}></div>
	{/if}

	<div class="flex h-[calc(100vh-73px)]">
		<!-- Left Sidebar -->
		<div
			data-left-sidebar
			class="flex flex-col border-r-2 transition-transform duration-300"
			class:fixed={isSmallScreen}
			class:left-0={isSmallScreen}
			class:top-0={isSmallScreen}
			class:h-full={isSmallScreen}
			class:z-40={isSmallScreen}
			class:w-80={isSmallScreen}
			class:w-72={!isSmallScreen}
			class:-translate-x-full={isSmallScreen && !leftSidebarOpen}
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div
				style="height: {leftPaletteHeight !== null
					? `${leftPaletteHeight}px`
					: '60%'}; flex-shrink: 0; overflow: auto;"
			>
				<TemplatesSidebar
					{templates}
					onCreateNew={createNewTemplate}
					onUseDefaultTemplate={useDefaultTemplate}
					onSelectTemplate={selectTemplate}
					currentTemplateName={isEditing ? (originalTemplateName ?? '') : logTitle}
					isNewTemplate={!isEditing}
				/>
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<div
				class="h-2 cursor-row-resize border-y-2 hover:bg-gray-100 dark:hover:bg-gray-700"
				style="border-color: var(--border-primary); flex-shrink: 0;"
				onmousedown={(e) => handleResizeStart(e, 'left')}
				ondblclick={() => (leftPaletteHeight = null)}
				role="separator"
				aria-orientation="horizontal"
			></div>
			<div class="flex-1 overflow-auto">
				<AiGeneratorSidebar
					bind:aiPrompt
					onGenerateLayout={generateLayoutFromPrompt}
					onUndoGeneration={undoGeneration}
					{aiLoading}
					{aiError}
					{hasUndoAvailable}
				/>
			</div>
		</div>

		<!-- Canvas Area -->
		<DesignCanvas
			bind:canvasItems
			bind:logTitle
			bind:versionName
			bind:branchId
			{branches}
			{canManageCompany}
			bind:selectedItemId
			bind:canvasRef
			onSave={saveTemplate}
			onDeleteSelected={deleteSelected}
			onDeleteTemplate={deleteTemplate}
			onItemMoved={markUnsavedChanges}
			onShowHistory={fetchVersions}
			{saving}
			{saveError}
			{saveSuccess}
			{loading}
			{isEditing}
			{deleting}
			{deleteError}
		/>

		<!-- Right Sidebar -->
		<div
			data-right-sidebar
			class="flex flex-col border-l-2 transition-transform duration-300"
			class:fixed={isSmallScreen}
			class:right-0={isSmallScreen}
			class:top-0={isSmallScreen}
			class:h-full={isSmallScreen}
			class:z-40={isSmallScreen}
			class:w-80={isSmallScreen}
			class:w-72={!isSmallScreen}
			class:translate-x-full={isSmallScreen && !rightSidebarOpen}
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div
				style="height: {rightPaletteHeight !== null
					? `${rightPaletteHeight}px`
					: '35%'}; flex-shrink: 0; overflow: auto;"
			>
				<ComponentsPalette {componentTypes} onAddComponent={addComponent} />
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<div
				class="h-2 cursor-row-resize border-y-2 hover:bg-gray-100 dark:hover:bg-gray-700"
				style="border-color: var(--border-primary); flex-shrink: 0;"
				onmousedown={(e) => handleResizeStart(e, 'right')}
				ondblclick={() => (rightPaletteHeight = null)}
				role="separator"
				aria-orientation="horizontal"
			></div>
			<div class="flex-1 overflow-auto">
				<PropertiesPanel {selectedItem} onUpdateProp={updateItemProp} onAlign={alignItem} />
			</div>
		</div>
	</div>
</div>

<style>
	.menu-bar {
		background-color: var(--bg-primary);
		border: 2px solid var(--border-primary);
		backdrop-filter: blur(10px);
		display: flex;
		align-items: center;
	}

	.menu-btn {
		color: var(--text-primary);
		transition: all 0.2s ease;
		font-weight: 500;
		white-space: nowrap;
		display: flex;
		align-items: center;
	}

	.menu-btn:hover {
		background-color: var(--bg-secondary);
	}

	.menu-btn-active {
		background-color: var(--bg-secondary);
	}

	.menu-btn:active {
		opacity: 0.8;
	}

	.menu-divider {
		width: 2px;
		height: 24px;
		background-color: var(--border-primary);
		margin: 0 0.5rem;
	}
</style>
