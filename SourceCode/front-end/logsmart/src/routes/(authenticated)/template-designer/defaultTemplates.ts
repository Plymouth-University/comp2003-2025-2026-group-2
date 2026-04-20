import type { components } from '$lib/api-types';

type ApiTemplateField = components['schemas']['TemplateField'];
type ApiSchedule = components['schemas']['Schedule'];

export type DefaultTemplateBlueprint = {
	id: string;
	name: string;
	description: string;
	category: 'checklist' | 'temperature' | 'combined';
	schedule: ApiSchedule;
	template_layout: ApiTemplateField[];
};

const WEEKDAY_DAILY: ApiSchedule = {
	frequency: 'Daily',
	days_of_week: [1, 2, 3, 4, 5]
};

const DAILY_ALL_DAYS: ApiSchedule = {
	frequency: 'Daily',
	days_of_week: [0, 1, 2, 3, 4, 5, 6]
};

export const DEFAULT_TEMPLATE_BLUEPRINTS: DefaultTemplateBlueprint[] = [
	{
		id: 'opening-checklist',
		name: 'Opening Shift Checklist',
		description:
			'Restaurant opening checks for hygiene stations, prep readiness, front-of-house setup, and manager sign-off.',
		category: 'checklist',
		schedule: { ...WEEKDAY_DAILY, available_from_time: '05:00', due_at_time: '10:00' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Opening Shift Checklist', size: '24', weight: 'bold' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 80 },
				props: { text: 'Handwash sinks stocked and functional', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 140 },
				props: {
					text: 'Sanitiser buckets prepared (correct dilution)',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 200 },
				props: {
					text: 'Delivery area checked and clear (no pest/activity signs)',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 260 },
				props: {
					text: 'High-risk food stock rotation completed (FIFO)',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 320 },
				props: {
					text: 'Front-of-house tables, menus, and booking notes prepared',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 390 },
				props: {
					text: 'Opening manager initials',
					placeholder: 'Enter initials',
					required: true,
					min_length: 2,
					max_length: 5,
					input_type: 'text'
				}
			}
		]
	},
	{
		id: 'fridge-freezer-temperature-log',
		name: 'Fridge & Freezer Temperature Log',
		description:
			'Restaurant chilled and frozen storage check with high-risk item verification and corrective actions.',
		category: 'temperature',
		schedule: { ...DAILY_ALL_DAYS, available_from_time: '06:00', due_at_time: '12:00' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Fridge & Freezer Temperature Log', size: '24', weight: 'bold' }
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 90 },
				props: { text: 'Fridge 1', min: 0, max: 8, unit: '°C', required: true, value: '0' }
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 162 },
				props: { text: 'Fridge 2', min: 0, max: 8, unit: '°C', required: true, value: '0' }
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 234 },
				props: {
					text: 'Walk-in freezer',
					min: -24,
					max: -12,
					unit: '°C',
					required: true,
					value: '-18'
				}
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 306 },
				props: {
					text: 'High-risk item sample (cooked meats/sauces)',
					min: 0,
					max: 5,
					unit: '°C',
					required: true,
					value: '4'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 378 },
				props: { text: 'Overall status', options: ['Pass', 'Action Needed'], selected: 'Pass' }
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 442 },
				props: {
					text: 'Corrective action notes',
					placeholder: 'E.g. moved stock, called engineer, discarded items',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			}
		]
	},
	{
		id: 'cleaning-verification-checklist',
		name: 'Cleaning Verification Checklist',
		description:
			'Restaurant close-down cleaning verification for food-contact areas, waste handling, and supervisor checks.',
		category: 'checklist',
		schedule: { ...DAILY_ALL_DAYS, available_from_time: '20:00', due_at_time: '23:59' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Cleaning Verification Checklist', size: '24', weight: 'bold' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 82 },
				props: { text: 'Food contact surfaces sanitised', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 142 },
				props: { text: 'Waste bins emptied and relined', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 202 },
				props: { text: 'Floors swept, mopped, and dry', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 262 },
				props: { text: 'Chemical storage secure', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 322 },
				props: {
					text: 'Dishwasher final rinse temperature recorded',
					required: false,
					selected: 'false'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 392 },
				props: {
					text: 'Issues found',
					placeholder: 'List missed areas, breakages, or follow-up tasks',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			}
		]
	},
	{
		id: 'cooking-temperature-verification',
		name: 'Cooking Temperature Verification',
		description:
			'Restaurant critical control point log for cook/reheat checks, probe hygiene, and batch traceability.',
		category: 'combined',
		schedule: { ...DAILY_ALL_DAYS, available_from_time: '10:00', due_at_time: '22:00' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Cooking Temperature Verification', size: '24', weight: 'bold' }
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 88 },
				props: {
					text: 'Dish / batch name',
					placeholder: 'e.g. Chicken Curry Batch A',
					required: true,
					max_length: 120,
					input_type: 'text'
				}
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 160 },
				props: {
					text: 'Core cooking temperature',
					min: 63,
					max: 100,
					unit: '°C',
					required: true,
					value: '75'
				}
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 232 },
				props: {
					text: 'Reheat temperature (if applicable)',
					min: 63,
					max: 100,
					unit: '°C',
					required: false,
					value: '0'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 304 },
				props: {
					text: 'Product type',
					options: ['Poultry', 'Minced meat', 'Whole cut meat', 'Seafood', 'Vegetarian'],
					selected: 'Poultry'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 370 },
				props: {
					text: 'Temperature probe cleaned and sanitised before use',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 436 },
				props: {
					text: 'Recorded by',
					placeholder: 'Name or initials',
					required: true,
					max_length: 60,
					input_type: 'text'
				}
			}
		]
	},
	{
		id: 'closing-shift-checklist',
		name: 'Closing Shift Checklist',
		description:
			'Restaurant closing routine for kitchen shutdown, stock security, cash-up handover, and alarm set.',
		category: 'checklist',
		schedule: { ...DAILY_ALL_DAYS, available_from_time: '20:00', due_at_time: '23:59' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Closing Shift Checklist', size: '24', weight: 'bold' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 82 },
				props: { text: 'Hot equipment switched off', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 142 },
				props: { text: 'Food covered and labelled', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 202 },
				props: {
					text: 'Waste removed from prep and service areas',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 262 },
				props: { text: 'Doors, windows, and storerooms secured', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 322 },
				props: {
					text: 'POS closed and end-of-day report completed',
					required: false,
					selected: 'false'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 392 },
				props: {
					text: 'Handover notes',
					placeholder: 'Maintenance issues, low stock, team notes',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			}
		]
	},
	{
		id: 'hot-holding-temperature-log',
		name: 'Hot Holding Temperature Log',
		description:
			'Restaurant service-period hot-hold checks for soups, bain-marie lines, and corrective action records.',
		category: 'temperature',
		schedule: { ...DAILY_ALL_DAYS, available_from_time: '11:00', due_at_time: '15:00' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Hot Holding Temperature Log', size: '24', weight: 'bold' }
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 88 },
				props: { text: 'Soup station', min: 63, max: 95, unit: '°C', required: true, value: '70' }
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 160 },
				props: { text: 'Main hot well', min: 63, max: 95, unit: '°C', required: true, value: '70' }
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 232 },
				props: {
					text: 'Secondary hot station',
					min: 63,
					max: 95,
					unit: '°C',
					required: true,
					value: '70'
				}
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 304 },
				props: {
					text: 'Carvery / pass counter',
					min: 63,
					max: 95,
					unit: '°C',
					required: false,
					value: '70'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 378 },
				props: {
					text: 'Any out-of-range values?',
					options: ['No', 'Yes - corrective action taken'],
					selected: 'No'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 442 },
				props: {
					text: 'Action taken',
					placeholder: 'Reheated above 75C, replaced batch, notified manager',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			}
		]
	},
	{
		id: 'receiving-temp-condition-check',
		name: 'Receiving Temperature & Condition Check',
		description:
			'Restaurant goods-in check for supplier traceability, delivery temperatures, packaging integrity, and acceptance decisions.',
		category: 'combined',
		schedule: { ...WEEKDAY_DAILY, available_from_time: '06:00', due_at_time: '13:00' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Receiving Temperature & Condition Check', size: '24', weight: 'bold' }
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 88 },
				props: {
					text: 'Supplier name and invoice number',
					placeholder: 'Enter supplier',
					required: true,
					max_length: 120,
					input_type: 'text'
				}
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 160 },
				props: {
					text: 'Chilled goods sample',
					min: 0,
					max: 8,
					unit: '°C',
					required: true,
					value: '5'
				}
			},
			{
				field_type: 'temperature',
				position: { x: 24, y: 232 },
				props: {
					text: 'Frozen goods sample',
					min: -24,
					max: -12,
					unit: '°C',
					required: false,
					value: '-18'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 304 },
				props: { text: 'Packaging intact and clean', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 364 },
				props: { text: 'Use-by dates acceptable', required: true, selected: 'false' }
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 424 },
				props: {
					text: 'Delivery decision',
					options: ['Accepted', 'Partially rejected', 'Rejected'],
					selected: 'Accepted'
				}
			}
		]
	},
	{
		id: 'allergen-control-checklist',
		name: 'Allergen Control Checklist',
		description:
			'Restaurant allergen control checks for menu accuracy, storage segregation, and team communication before service.',
		category: 'checklist',
		schedule: { ...DAILY_ALL_DAYS, available_from_time: '07:00', due_at_time: '11:00' },
		template_layout: [
			{
				field_type: 'label',
				position: { x: 24, y: 20 },
				props: { text: 'Allergen Control Checklist', size: '24', weight: 'bold' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 84 },
				props: { text: 'Allergen matrix is current', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 144 },
				props: {
					text: 'Segregated storage labels visible',
					required: true,
					selected: 'false'
				}
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 204 },
				props: { text: 'Dedicated utensils available', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 264 },
				props: { text: 'Front-of-house brief completed', required: true, selected: 'false' }
			},
			{
				field_type: 'checkbox',
				position: { x: 24, y: 324 },
				props: {
					text: 'Online ordering allergen notes reviewed',
					required: false,
					selected: 'false'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 394 },
				props: {
					text: 'Allergen incidents or near misses',
					placeholder: 'Record details and escalation actions',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			}
		]
	}
];
