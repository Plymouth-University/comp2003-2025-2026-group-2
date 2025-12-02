<script lang="ts">
	import { browser } from '$app/environment';
	import TemplatesSidebar from './TemplatesSidebar.svelte';
	import DesignCanvas from './DesignCanvas.svelte';
	import ComponentsPalette from './ComponentsPalette.svelte';
	import PropertiesPanel from './PropertiesPanel.svelte';
	import type { CanvasItem, ComponentType, Template } from './types';

	const templates: Template[] = [];

	const componentTypes: ComponentType[] = [
		{ type: 'text_input', name: 'Text Input', icon: 'T' },
		{ type: 'checkbox', name: 'Checkbox', icon: '✓' },
		{ type: 'temperature', name: 'Temperature Input', icon: '🌡' },
		{ type: 'dropdown', name: 'Dropdown', icon: '≡' },
		{ type: 'label', name: 'Text Label', icon: 'L' }
	];

	let canvasItems = $state<CanvasItem[]>(
		browser && localStorage.getItem('canvasItems')
			? JSON.parse(localStorage.getItem('canvasItems')!)
			: []
	);

	let logTitle = $state(
		browser && localStorage.getItem('logTitle') ? localStorage.getItem('logTitle')! : ''
	);

	let selectedItemId = $state<string | null>(null);

	// Debounced localStorage saves to prevent lag during dragging
	let saveTimeout: ReturnType<typeof setTimeout> | null = null;
	function debouncedSaveItems() {
		if (saveTimeout) clearTimeout(saveTimeout);
		saveTimeout = setTimeout(() => {
			if (browser) {
				localStorage.setItem('canvasItems', JSON.stringify(canvasItems));
			}
		}, 300);
	}

	$effect(() => {
		// Deep track canvasItems by serializing it
		const serialized = JSON.stringify(canvasItems);
		debouncedSaveItems();
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem('logTitle', logTitle);
		}
	});

	function createNewTemplate() {
		canvasItems = [];
		logTitle = '';
		selectedItemId = null;
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
	}

	function deleteSelected() {
		if (selectedItemId) {
			canvasItems = canvasItems.filter((item) => item.id !== selectedItemId);
			selectedItemId = null;
		}
	}

	function exportToJson() {
		const exportData = {
			title: logTitle,
			items: canvasItems.map((item) => ({
				type: item.type,
				x: item.x,
				y: item.y,
				props: item.props
			}))
		};
		console.log('Exported Template JSON:', JSON.stringify(exportData, null, 2));
	}

	function updateItemProp(itemId: string, propKey: string, value: any) {
		// Handle top-level properties like lockX and lockY
		if (propKey === 'lockX' || propKey === 'lockY') {
			canvasItems = canvasItems.map((item) =>
				item.id === itemId ? { ...item, [propKey]: value } : item
			);
		} else {
			canvasItems = canvasItems.map((item) =>
				item.id === itemId ? { ...item, props: { ...item.props, [propKey]: value } } : item
			);
		}
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
</script>

<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<div class="flex h-[calc(100vh-73px)]">
		<TemplatesSidebar {templates} onCreateNew={createNewTemplate} />

		<DesignCanvas
			bind:canvasItems
			bind:logTitle
			bind:selectedItemId
			bind:canvasRef
			onExport={exportToJson}
			onDeleteSelected={deleteSelected}
		/>

		<div
			data-right-sidebar
			class="flex w-72 flex-col border-l-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<div
				style="height: {paletteHeight !== null
					? `${paletteHeight}px`
					: '50%'}; flex-shrink: 0; overflow: auto;"
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
