import type { components } from '$lib/api-types';

type ApiTemplateField = components['schemas']['TemplateField'];
type ApiSchedule = components['schemas']['Schedule'];

export type DefaultTemplateBlueprint = {
	id: string;
	name: string;
	description: string;
	category: 'checklist' | 'temperature' | 'combined';
	canvas_height: number;
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

const BASE_DEFAULT_TEMPLATE_BLUEPRINTS: DefaultTemplateBlueprint[] = [
	{
		id: 'opening-checklist',
		name: 'Opening Shift Checklist',
		description:
			'Restaurant opening checks for hygiene stations, prep readiness, front-of-house setup, and manager sign-off.',
		category: 'checklist',
		canvas_height: 1060,
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
					text: 'Opening manager initials (person completing this checklist)',
					placeholder: 'Enter initials, e.g. JD',
					required: true,
					min_length: 2,
					max_length: 5,
					input_type: 'text'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 460 },
				props: {
					text: 'Opening readiness decision after all checks',
					options: ['Ready to open', 'Not ready - escalation required'],
					selected: 'Ready to open'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 530 },
				props: {
					text: 'Escalation notes and owner (if not ready)',
					placeholder: 'State issue, who owns it, and expected resolution time',
					required: false,
					max_length: 250,
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
		canvas_height: 1140,
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
				props: {
					text: 'Overall storage compliance status after all readings',
					options: ['Compliant - all within limits', 'Non-compliant - corrective action required'],
					selected: 'Compliant - all within limits'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 442 },
				props: {
					text: 'Corrective action notes (if any reading is out of range)',
					placeholder:
						'Describe action, time, and who was informed. E.g. moved stock, called engineer, discarded items',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 506 },
				props: {
					text: 'Checked by (name or initials)',
					placeholder: 'Enter checker name/initials',
					required: true,
					max_length: 60,
					input_type: 'text'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 570 },
				props: {
					text: 'Probe calibration check before use',
					options: ['Probe calibrated/verified', 'Probe not verified - use backup probe'],
					selected: 'Probe calibrated/verified'
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
		canvas_height: 1020,
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
					text: 'Issues found during close-down clean',
					placeholder:
						'List missed areas, breakages, chemical shortages, or follow-up tasks for next shift',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 462 },
				props: {
					text: 'Cleaning verification outcome for this shift',
					options: ['Complete - standards met', 'Incomplete - follow-up required'],
					selected: 'Complete - standards met'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 526 },
				props: {
					text: 'Verified by (supervisor/manager)',
					placeholder: 'Enter verifier name/initials',
					required: true,
					max_length: 60,
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
		canvas_height: 1040,
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
					text: 'Product category being verified',
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
					text: 'Recorded by (name or initials of checker)',
					placeholder: 'Enter full name or initials',
					required: true,
					max_length: 60,
					input_type: 'text'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 500 },
				props: {
					text: 'CCP outcome for this batch',
					options: ['Pass - safe to serve', 'Fail - hold/dispose and escalate'],
					selected: 'Pass - safe to serve'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 566 },
				props: {
					text: 'Corrective action and escalation details (if CCP failed)',
					placeholder: 'Describe hold/dispose action, manager notified, and next step',
					required: false,
					max_length: 250,
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
		canvas_height: 1020,
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
					text: 'End-of-shift handover notes for next team',
					placeholder:
						'Include maintenance issues, low stock, incidents, and opening priorities for next shift',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 462 },
				props: {
					text: 'Site security confirmation before leaving',
					options: ['Alarm set and premises secure', 'Security incomplete - action required'],
					selected: 'Alarm set and premises secure'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 526 },
				props: {
					text: 'Closed by (shift lead name/initials)',
					placeholder: 'Enter closing lead name/initials',
					required: true,
					max_length: 60,
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
		canvas_height: 1080,
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
					text: 'Were any hot-hold readings below 63C during this check?',
					options: ['No - all readings compliant', 'Yes - corrective action taken'],
					selected: 'No - all readings compliant'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 442 },
				props: {
					text: 'Corrective action details (required if any station below 63C)',
					placeholder:
						'Record station, reading, action taken, recheck result, and manager notified',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 506 },
				props: {
					text: 'Checked by (service lead name/initials)',
					placeholder: 'Enter checker name/initials',
					required: true,
					max_length: 60,
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
		canvas_height: 1040,
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
					text: 'Supplier name and delivery note/invoice reference',
					placeholder: 'E.g. FreshFoods Ltd - INV-18429',
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
					text: 'Delivery acceptance decision after checks',
					options: ['Accepted', 'Partially rejected', 'Rejected'],
					selected: 'Accepted'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 488 },
				props: {
					text: 'Goods received by (name/initials)',
					placeholder: 'Enter receiving staff name/initials',
					required: true,
					max_length: 60,
					input_type: 'text'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 552 },
				props: {
					text: 'Rejected/returned items and reason (if applicable)',
					placeholder: 'Item, quantity, reason, supplier informed, and follow-up action',
					required: false,
					max_length: 250,
					input_type: 'text'
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
		canvas_height: 1020,
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
					text: 'Allergen incidents, near misses, or escalations',
					placeholder:
						'Record what happened, affected item, immediate actions, and manager escalation',
					required: false,
					max_length: 250,
					input_type: 'text'
				}
			},
			{
				field_type: 'dropdown',
				position: { x: 24, y: 464 },
				props: {
					text: 'Any menu/process changes requiring allergen update today?',
					options: ['No changes', 'Yes - allergen info updated and briefed'],
					selected: 'No changes'
				}
			},
			{
				field_type: 'text_input',
				position: { x: 24, y: 528 },
				props: {
					text: 'Team brief completed by (name/initials)',
					placeholder: 'Enter person responsible for allergen brief',
					required: true,
					max_length: 60,
					input_type: 'text'
				}
			}
		]
	}
];

function getTopicAwareFieldText(
	field: ApiTemplateField,
	fieldIndexByType: Record<string, number>,
	templateId: string
): string {
	const currentCount = (fieldIndexByType[field.field_type] ?? 0) + 1;
	fieldIndexByType[field.field_type] = currentCount;
	const existingText = typeof field.props?.text === 'string' ? field.props.text.trim() : '';

	// Preserve template-specific wording wherever possible to avoid overgeneralizing.
	if (field.field_type !== 'temperature' && existingText.length > 0) {
		return existingText;
	}

	const topicByTemplate: Record<string, string> = {
		'opening-checklist': 'Opening',
		'fridge-freezer-temperature-log': 'Storage',
		'cleaning-verification-checklist': 'Cleaning',
		'cooking-temperature-verification': 'Cooking',
		'closing-shift-checklist': 'Closing',
		'hot-holding-temperature-log': 'Hot holding',
		'receiving-temp-condition-check': 'Receiving',
		'allergen-control-checklist': 'Allergen'
	};

	const topic = topicByTemplate[templateId] ?? 'Template';

	switch (field.field_type) {
		case 'temperature':
			if (templateId === 'hot-holding-temperature-log') {
				return `Hot plate ${currentCount}`;
			}
			if (existingText.length > 0) {
				return existingText;
			}
			return `${topic} temperature check ${currentCount}`;
		case 'dropdown':
			return `${topic} status ${currentCount}`;
		case 'checkbox':
			return `${topic} checklist item ${currentCount}`;
		case 'text':
		case 'text_input': {
			const required = field.props?.required === true || field.props?.required === 'true';
			return required
				? `${topic} required details ${currentCount}`
				: `${topic} notes ${currentCount}`;
		}
		default:
			return `Field ${currentCount}`;
	}
}

function createPromptLabelForField(field: ApiTemplateField, text: string): ApiTemplateField {
	return {
		field_type: 'label',
		position: { x: 24, y: 24 },
		props: {
			text,
			size: '14',
			weight: 'bold'
		}
	};
}

function withGenericComponentText(field: ApiTemplateField, text: string): ApiTemplateField {
	if (field.field_type === 'label') return field;

	return {
		...field,
		props: {
			...field.props,
			text
		}
	};
}

function getFieldRenderHeight(field: ApiTemplateField): number {
	switch (field.field_type) {
		case 'temperature':
			return 84;
		case 'text_input':
		case 'text':
			return 52;
		case 'dropdown':
			return 44;
		case 'checkbox':
			return 34;
		case 'label':
			return 26;
		default:
			return 44;
	}
}

function getEstimatedLabelHeight(text: string): number {
	const trimmed = text.trim();
	if (!trimmed) return 24;

	// Labels wrap in the canvas; estimate the number of lines for stable spacing.
	const approxCharsPerLine = 44;
	const estimatedLines = Math.max(1, Math.ceil(trimmed.length / approxCharsPerLine));
	return 24 + (estimatedLines - 1) * 18;
}

function buildSpacedTemplateLayout(fields: ApiTemplateField[], templateId: string): ApiTemplateField[] {
	const spaced: ApiTemplateField[] = [];
	let cursorY = 20;
	const fieldIndexByType: Record<string, number> = {};

	const LABEL_TO_FIELD_GAP = 14;
	const GROUP_GAP = 20;

	fields.forEach((field, index) => {
		const x = field.position?.x ?? 24;

		if (field.field_type === 'label') {
			const isTitle = index === 0;
			spaced.push({
				...field,
				position: { x, y: cursorY }
			});
			cursorY += isTitle ? 56 : 32;
			return;
		}

		const topicText = getTopicAwareFieldText(field, fieldIndexByType, templateId);
		const promptLabel = createPromptLabelForField(field, topicText);
		const genericField = withGenericComponentText(field, topicText);
		spaced.push({
			...promptLabel,
			position: { x, y: cursorY }
		});
		cursorY += getEstimatedLabelHeight(topicText) + LABEL_TO_FIELD_GAP;

		spaced.push({
			...genericField,
			position: { x, y: cursorY }
		});
		cursorY += getFieldRenderHeight(genericField) + GROUP_GAP;
	});

	return spaced;
}

export const DEFAULT_TEMPLATE_BLUEPRINTS: DefaultTemplateBlueprint[] =
	BASE_DEFAULT_TEMPLATE_BLUEPRINTS.map((blueprint) => ({
		...blueprint,
		template_layout: buildSpacedTemplateLayout(blueprint.template_layout, blueprint.id)
	}));
