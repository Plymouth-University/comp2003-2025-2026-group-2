export type ScheduleFrequency = 'daily' | 'weekly' | 'monthly' | 'yearly' | 'custom';

export type DayOfWeek =
	| 'monday'
	| 'tuesday'
	| 'wednesday'
	| 'thursday'
	| 'friday'
	| 'saturday'
	| 'sunday';

export interface TemplateSchedule {
	frequency: ScheduleFrequency;
	daysOfWeek?: DayOfWeek[];
	dayOfWeek?: DayOfWeek;
	dayOfMonth?: number;
	monthOfYear?: number;
	customIntervalDays?: number;
}

export interface Template {
	id: string;
	name: string;
	createdAt: string;
	updatedAt: string;
	schedule: TemplateSchedule;
	layout: CanvasItem[];
}

export interface CanvasItem {
	id: string;
	type: 'text_input' | 'label' | 'checkbox' | 'dropdown' | 'temperature_picker';
	x: number;
	y: number;
	lockX?: boolean;
	lockY?: boolean;
	props: Record<string, unknown>;
}
