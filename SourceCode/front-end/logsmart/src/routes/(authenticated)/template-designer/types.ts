export type CanvasItem = {
	id: string;
	type: string;
	x: number;
	y: number;
	lockX?: boolean;
	lockY?: boolean;
	props: Record<string, any>;
};

export type ComponentType = {
	type: string;
	name: string;
	icon: string;
};

export type Template = {
	id: number;
	name: string;
	selected: boolean;
};
