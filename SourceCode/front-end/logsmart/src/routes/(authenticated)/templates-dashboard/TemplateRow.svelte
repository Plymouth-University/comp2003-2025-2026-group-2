<script lang="ts">
	import type { Template, TemplateSchedule, DayOfWeek } from './types';

	let {
		template,
		onEdit,
		onSettings,
		onDelete
	}: {
		template: Template;
		onEdit: (template: Template) => void;
		onSettings: (template: Template) => void;
		onDelete: (template: Template) => void;
	} = $props();

	function formatSchedule(schedule: TemplateSchedule): string {
		switch (schedule.frequency) {
			case 'daily':
				if (schedule.daysOfWeek && schedule.daysOfWeek.length > 0) {
					const dayLabels: Record<DayOfWeek, string> = {
						monday: 'Mon',
						tuesday: 'Tue',
						wednesday: 'Wed',
						thursday: 'Thu',
						friday: 'Fri',
						saturday: 'Sat',
						sunday: 'Sun'
					};
					return `Daily (${schedule.daysOfWeek.map((d) => dayLabels[d]).join(', ')})`;
				}
				return 'Daily';
			case 'weekly':
				if (schedule.dayOfWeek) {
					const dayLabels: Record<DayOfWeek, string> = {
						monday: 'Monday',
						tuesday: 'Tuesday',
						wednesday: 'Wednesday',
						thursday: 'Thursday',
						friday: 'Friday',
						saturday: 'Saturday',
						sunday: 'Sunday'
					};
					return `Weekly (${dayLabels[schedule.dayOfWeek]})`;
				}
				return 'Weekly';
			case 'monthly':
				return `Monthly (Day ${schedule.dayOfMonth})`;
			case 'yearly':
				const months = [
					'Jan',
					'Feb',
					'Mar',
					'Apr',
					'May',
					'Jun',
					'Jul',
					'Aug',
					'Sep',
					'Oct',
					'Nov',
					'Dec'
				];
				return `Yearly (${months[(schedule.monthOfYear || 1) - 1]} ${schedule.dayOfMonth})`;
			case 'custom':
				return `Every ${schedule.customIntervalDays} days`;
			default:
				return 'Not set';
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-GB', {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		});
	}
</script>

<div
	class="flex flex-col gap-3 rounded-lg border-2 px-4 py-4 transition-colors hover:shadow-md sm:flex-row sm:items-center sm:justify-between sm:px-6"
	style="background-color: var(--bg-primary); border-color: var(--border-primary);"
>
	<div class="min-w-0 flex-1">
		<h3 class="truncate text-base font-bold sm:text-lg" style="color: var(--text-primary);">
			{template.name}
		</h3>
		<div
			class="mt-1 flex flex-wrap gap-2 text-xs sm:gap-4 sm:text-sm"
			style="color: var(--text-secondary);"
		>
			<span class="truncate">Schedule: {formatSchedule(template.schedule)}</span>
			<span class="hidden sm:inline">•</span>
			<span class="truncate">Updated: {formatDate(template.updatedAt)}</span>
		</div>
	</div>

	<div class="flex w-full shrink-0 gap-1 sm:w-auto sm:gap-2">
		<button
			type="button"
			class="btn-edit flex-1 rounded px-2 py-2 text-xs font-medium text-white sm:flex-none sm:px-4 sm:text-base"
			onclick={() => onEdit(template)}
			title="Edit template design"
		>
			<span class="hidden xs:inline">✏️ Edit</span>
			<span class="xs:hidden">✏️</span>
		</button>
		<button
			type="button"
			class="btn-settings flex-1 rounded px-2 py-2 text-xs font-medium text-white sm:flex-none sm:px-4 sm:text-base"
			onclick={() => onSettings(template)}
			title="Schedule settings"
		>
			<span class="hidden xs:inline">⚙️ Settings</span>
			<span class="xs:hidden">⚙️</span>
		</button>
		<button
			type="button"
			class="btn-delete flex-none rounded px-2 py-2 text-xs font-medium text-white sm:px-4 sm:text-base"
			onclick={() => onDelete(template)}
			title="Delete template"
		>
			🗑️
		</button>
	</div>
</div>

<style>
	.btn-edit {
		background-color: #337ab7;
		transition: background-color 0.15s ease;
	}

	.btn-edit:hover {
		background-color: #286090;
	}

	.btn-edit:active {
		background-color: #204d74;
	}

	.btn-settings {
		background-color: #5bc0de;
		transition: background-color 0.15s ease;
	}

	.btn-settings:hover {
		background-color: #31b0d5;
	}

	.btn-settings:active {
		background-color: #269abc;
	}

	.btn-delete {
		background-color: #d9534f;
		transition: background-color 0.15s ease;
	}

	.btn-delete:hover {
		background-color: #c9302c;
	}

	.btn-delete:active {
		background-color: #ac2925;
	}
</style>
