<script lang="ts">
	import { browser } from '$app/environment';
	import TemplatesSidebar from './TemplatesSidebar.svelte';
	import DesignCanvas from './DesignCanvas.svelte';
	import ComponentsPalette from './ComponentsPalette.svelte';
	import PropertiesPanel from './PropertiesPanel.svelte';
	import type { CanvasItem, ComponentType, Template } from './types';

	const templates: Template[] = [
		{ id: 1, name: 'Kitchen Daily Log', selected: false },
		{ id: 2, name: 'Kitchen Cleaning Log', selected: false },
		{ id: 3, name: 'Bar Log', selected: false }
	];

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

	$effect(() => {
		if (browser) {
			localStorage.setItem('canvasItems', JSON.stringify(canvasItems));
		}
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

	function handleDragStart(e: DragEvent, type: string) {
		if (e.dataTransfer) {
			e.dataTransfer.setData('component-type', type);
			e.dataTransfer.effectAllowed = 'copy';
		}
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
		canvasItems = canvasItems.map((item) =>
			item.id === itemId ? { ...item, props: { ...item.props, [propKey]: value } } : item
		);
	}

	let selectedItem = $derived(canvasItems.find((item) => item.id === selectedItemId));
</script>

<div class="min-h-full" style="background-color: var(--bg-secondary);">
	<div class="flex h-[calc(100vh-73px)]">
		<TemplatesSidebar {templates} onCreateNew={createNewTemplate} />

		<DesignCanvas
			bind:canvasItems
			bind:logTitle
			bind:selectedItemId
			onExport={exportToJson}
			onDeleteSelected={deleteSelected}
		/>

		<div
			class="flex w-72 flex-col border-l-2"
			style="border-color: var(--border-primary); background-color: var(--bg-primary);"
		>
			<ComponentsPalette {componentTypes} onDragStart={handleDragStart} />
			<PropertiesPanel {selectedItem} onUpdateProp={updateItemProp} />
		</div>
	</div>
</div>
