export interface SnapResult {
	x: number;
	y: number;
	snapLinesX: number[];
	snapLinesY: number[];
}

export interface Bounds {
	left: number;
	right: number;
	top: number;
	bottom: number;
	centerX: number;
	centerY: number;
	width: number;
	height: number;
}

export function getElementBounds(el: HTMLElement, canvasRef: HTMLElement): Bounds {
	const canvasRect = canvasRef.getBoundingClientRect();
	const itemRect = el.getBoundingClientRect();
	const left = itemRect.left - canvasRect.left;
	const top = itemRect.top - canvasRect.top;
	return {
		left,
		right: left + itemRect.width,
		top,
		bottom: top + itemRect.height,
		width: itemRect.width,
		height: itemRect.height,
		centerX: left + itemRect.width / 2,
		centerY: top + itemRect.height / 2
	};
}

export function calculateSnap(
	draggedId: string,
	proposedX: number,
	proposedY: number,
	canvasRef: HTMLElement,
	otherItemIds: string[],
	threshold: number = 10
): SnapResult {
	const draggedEl = document.querySelector(`[data-item-id="${draggedId}"]`) as HTMLElement;
	if (!draggedEl || !canvasRef) {
		return { x: proposedX, y: proposedY, snapLinesX: [], snapLinesY: [] };
	}

	const draggedRect = draggedEl.getBoundingClientRect();
	const width = draggedRect.width;
	const height = draggedRect.height;

	const draggedLeft = proposedX;
	const draggedRight = proposedX + width;
	const draggedCenterX = proposedX + width / 2;
	const draggedTop = proposedY;
	const draggedBottom = proposedY + height;
	const draggedCenterY = proposedY + height / 2;

	let bestSnapX: { snappedX: number; distance: number; line: number } | null = null;
	let bestSnapY: { snappedY: number; distance: number; line: number } | null = null;

	const snapLinesX: number[] = [];
	const snapLinesY: number[] = [];

	for (const itemId of otherItemIds) {
		const el = document.querySelector(`[data-item-id="${itemId}"]`) as HTMLElement;
		if (!el) continue;

		const bounds = getElementBounds(el, canvasRef);

		const xPoints = [
			{ dragPoint: draggedLeft, snapTo: bounds.left, line: bounds.left },
			{ dragPoint: draggedLeft, snapTo: bounds.right, line: bounds.right },
			{ dragPoint: draggedRight, snapTo: bounds.left, line: bounds.left },
			{ dragPoint: draggedRight, snapTo: bounds.right, line: bounds.right },
			{ dragPoint: draggedCenterX, snapTo: bounds.centerX, line: bounds.centerX },
			{ dragPoint: draggedLeft, snapTo: bounds.centerX, line: bounds.centerX },
			{ dragPoint: draggedRight, snapTo: bounds.centerX, line: bounds.centerX },
			{ dragPoint: draggedCenterX, snapTo: bounds.left, line: bounds.left },
			{ dragPoint: draggedCenterX, snapTo: bounds.right, line: bounds.right }
		];

		for (const { dragPoint, snapTo, line } of xPoints) {
			const distance = Math.abs(dragPoint - snapTo);
			if (distance < threshold) {
				if (!bestSnapX || distance < bestSnapX.distance) {
					const offset = dragPoint - draggedLeft;
					bestSnapX = { snappedX: snapTo - offset, distance, line };
				}
			}
		}

		const yPoints = [
			{ dragPoint: draggedTop, snapTo: bounds.top, line: bounds.top },
			{ dragPoint: draggedTop, snapTo: bounds.bottom, line: bounds.bottom },
			{ dragPoint: draggedBottom, snapTo: bounds.top, line: bounds.top },
			{ dragPoint: draggedBottom, snapTo: bounds.bottom, line: bounds.bottom },
			{ dragPoint: draggedCenterY, snapTo: bounds.centerY, line: bounds.centerY },
			{ dragPoint: draggedTop, snapTo: bounds.centerY, line: bounds.centerY },
			{ dragPoint: draggedBottom, snapTo: bounds.centerY, line: bounds.centerY },
			{ dragPoint: draggedCenterY, snapTo: bounds.top, line: bounds.top },
			{ dragPoint: draggedCenterY, snapTo: bounds.bottom, line: bounds.bottom }
		];

		for (const { dragPoint, snapTo, line } of yPoints) {
			const distance = Math.abs(dragPoint - snapTo);
			if (distance < threshold) {
				if (!bestSnapY || distance < bestSnapY.distance) {
					const offset = dragPoint - draggedTop;
					bestSnapY = { snappedY: snapTo - offset, distance, line };
				}
			}
		}
	}

	const finalX = bestSnapX ? bestSnapX.snappedX : proposedX;
	const finalY = bestSnapY ? bestSnapY.snappedY : proposedY;

	if (bestSnapX) snapLinesX.push(bestSnapX.line);
	if (bestSnapY) snapLinesY.push(bestSnapY.line);

	return { x: finalX, y: finalY, snapLinesX, snapLinesY };
}
