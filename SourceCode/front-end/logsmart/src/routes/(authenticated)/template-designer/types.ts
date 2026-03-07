export type CanvasItemProps = {
	text?: string;
	placeholder?: string;
	size?: number;
	weight?: string;
	editable?: boolean;
	min?: number;
	max?: number;
	value?: string | number;
	unit?: string;
	options?: string[];
	selected?: string | number | boolean;
	fontFamily?: string;
	textDecoration?: string;
	color?: string;
	required?: boolean;
	maxLength?: number | null;
	minLength?: number | null;
	inputType?: string;
	label?: string;
};

export type CanvasItem = {
	id: string;
	type: string;
	x: number;
	y: number;
	lockX?: boolean;
	lockY?: boolean;
	props: CanvasItemProps;
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
