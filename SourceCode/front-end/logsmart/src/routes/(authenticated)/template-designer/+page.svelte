<script lang="ts">
	import { browser } from '$app/environment';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import type { components } from '$lib/api-types';
	import TemplatesSidebar from './TemplatesSidebar.svelte';
	import DesignCanvas from './DesignCanvas.svelte';
	import ComponentsPalette from './ComponentsPalette.svelte';
	import PropertiesPanel from './PropertiesPanel.svelte';
	import AiGeneratorSidebar from './AiGeneratorSidebar.svelte';
	import VersionHistoryModal from './VersionHistoryModal.svelte';
	import type { CanvasItem, ComponentType, Template } from './types';

	type ApiTemplateField = components['schemas']['TemplateField'];
	type ApiTemplateFieldProps = components['schemas']['TemplateFieldProps'];
	type ApiSchedule = components['schemas']['Schedule'];
	type ApiTemplateInfo = components['schemas']['TemplateInfo'];
	type ApiTemplateVersionInfo = components['schemas']['TemplateVersionInfo'];

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
	let deleting = $state(false);
	let deleteError = $state<string | null>(null);
	let hasUnsavedChanges = $state(false);
	let aiPrompt = $state('');
	let aiLoading = $state(false);
	let aiError = $state<string | null>(null);
	let canvasItemsBackup = $state<CanvasItem[] | null>(null);
	let hasUndoAvailable = $derived(canvasItemsBackup !== null);

	let showHistory = $state(false);
	let historyVersions = $state<ApiTemplateVersionInfo[]>([]);
	let restoring = $state(false);

	const templateId = $derived(page.url.searchParams.get('id'));

	function mapApiFieldToCanvasItem(field: ApiTemplateField, index: number): CanvasItem {
		return {
			id: `item_${index}_${Math.random().toString(36).substring(2, 9)}`,
			type: field.field_type,
			x: field.position.x,
			y: field.position.y,
			props: {
				text: field.props.text ?? '',
				placeholder: field.props.placeholder ?? '',
				size: field.props.size ?? 16,
				weight: field.props.weight ?? 'normal',
				editable: field.props.editable ?? true,
				min: field.props.min ?? 0,
				max: field.props.max ?? 100,
				value: field.props.value ?? '',
				unit: field.props.unit ?? '°C',
				options: field.props.options ?? [],
				selected: field.props.selected ?? false
			}
		};
	}

	function mapCanvasItemToApiField(item: CanvasItem): ApiTemplateField {
		const props: ApiTemplateFieldProps = {};

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

		return {
			field_type: item.type,
			position: { x: item.x, y: item.y },
			props
		};
	}

	async function loadTemplate(name: string) {
		loading = true;
		try {
			const response = await fetch(`/api/logs/templates?template_name=${encodeURIComponent(name)}`);
			if (!response.ok) {
				console.error('Failed to load template:', response.statusText);
				loading = false;
				return;
			}
			const data = (await response.json()) as components['schemas']['GetTemplateResponse'];
			logTitle = data.template_name;
			originalTemplateName = data.template_name;
			canvasItems = data.template_layout.map(mapApiFieldToCanvasItem);
			hasUnsavedChanges = false;
			isEditing = true;
		} catch (e) {
			console.error('Failed to load template:', e);
		}
		loading = false;
	}

	async function saveTemplate() {
		if (!logTitle.trim()) {
			saveError = 'Please enter a template name';
			return;
		}

		saving = true;
		saveError = null;
		saveSuccess = false;

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
					template_layout: templateLayout
				}
			});

			if (error) {
				saveError = 'Failed to update template';
				saving = false;
				return;
			}
		} else {
			const schedule: ApiSchedule = {
				frequency: 'Daily',
				days_of_week: [1, 2, 3, 4, 5]
			};

			const { error } = await api.POST('/logs/templates', {
				body: {
					template_name: logTitle,
					template_layout: templateLayout,
					schedule
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

	$effect(() => {
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
			templates = data.templates.map((t: ApiTemplateInfo, index: number) => ({
				id: index,
				name: t.template_name,
				selected: t.template_name === originalTemplateName
			}));
		}
	}

	$effect(() => {
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
		hasUnsavedChanges = false;
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

	function getDefaultProps(type: string): Record<string, any> {
		switch (type) {
			case 'text_input':
				return { text: '', size: 16, weight: 'normal' };
			case 'checkbox':
				return { text: 'Checkbox Label', size: '16px', weight: 'normal' };
			case 'temperature':
				return { value: 0, min: -20, max: 50, label: 'Temperature', unit: '°C' };
			case 'dropdown':
				return { selected: '', options: ['Option 1', 'Option 2', 'Option 3'] };
			case 'label':
				return { editable: true, text: 'Label Text', size: 16, weight: 'normal' };
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

	function updateItemProp(itemId: string, propKey: string, value: any) {
		if (propKey === 'lockX' || propKey === 'lockY' || propKey === 'x' || propKey === 'y') {
			canvasItems = canvasItems.map((item) =>
				item.id === itemId ? { ...item, [propKey]: value } : item
			);
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

	let paletteHeight = $state<number | null>(null);
	let isResizing = $state(false);

	function handleResizeStart(e: MouseEvent) {
		e.preventDefault();
		isResizing = true;
		window.addEventListener('mousemove', handleResizeMove);
		window.addEventListener('mouseup', handleResizeEnd);
	}

	function handleResizeMove(e: MouseEvent) {
		if (!isResizing) return;
		const sidebar = document.querySelector('[data-right-sidebar]');
		if (sidebar) {
			const sidebarRect = sidebar.getBoundingClientRect();
			if (paletteHeight === null) {
				paletteHeight = sidebarRect.height / 2;
			}
			const newHeight = e.clientY - sidebarRect.top;
			paletteHeight = Math.max(100, Math.min(newHeight, sidebarRect.height - 100));
		}
	}

	function handleResizeEnd() {
		isResizing = false;
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

		restoring = true;
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
			restoring = false;
			return;
		}

		// Reload the template to reflect changes
		await loadTemplate(originalTemplateName);
		showHistory = false;
		restoring = false;
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
	onClose={() => (showHistory = false)}
	onRestore={restoreVersion}
/>

<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<div class="flex h-[calc(100vh-73px)]">
		<div
			data-left-sidebar
			class="flex w-72 flex-col border-r-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div
				style="height: {paletteHeight !== null
					? `${paletteHeight}px`
					: '60%'}; flex-shrink: 0; overflow: auto;"
			>
				<TemplatesSidebar
					{templates}
					onCreateNew={createNewTemplate}
					onSelectTemplate={selectTemplate}
					currentTemplateName={isEditing ? (originalTemplateName ?? '') : logTitle}
					isNewTemplate={!isEditing}
				/>
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<div
				class="h-2 cursor-row-resize border-y-2 hover:bg-gray-200"
				style="border-color: var(--border-primary); flex-shrink: 0;"
				onmousedown={handleResizeStart}
				ondblclick={() => (paletteHeight = null)}
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
		<DesignCanvas
			bind:canvasItems
			bind:logTitle
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

		<div
			data-right-sidebar
			class="flex w-72 flex-col border-l-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div
				style="height: {paletteHeight !== null
					? `${paletteHeight}px`
					: '35%'}; flex-shrink: 0; overflow: auto;"
			>
				<ComponentsPalette {componentTypes} onAddComponent={addComponent} />
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<div
				class="h-2 cursor-row-resize border-y-2 hover:bg-gray-200"
				style="border-color: var(--border-primary); flex-shrink: 0;"
				onmousedown={handleResizeStart}
				ondblclick={() => (paletteHeight = null)}
				role="separator"
				aria-orientation="horizontal"
			></div>
			<div class="flex-1 overflow-auto">
				<PropertiesPanel {selectedItem} onUpdateProp={updateItemProp} onAlign={alignItem} />
			</div>
		</div>
	</div>
</div>
