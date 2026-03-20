export function generateAttendancePdfHtml(params: {
	companyName: string;
	dateRange: string;
	events: Array<{
		first_name: string;
		last_name: string;
		email: string;
		clock_in: string;
		clock_out: string | null;
		status: string;
	}>;
	formatDateTime: (iso: string) => string;
	formatDuration: (clockIn: string, clockOut: string | null) => string;
}): string {
	const { companyName, dateRange, events, formatDateTime, formatDuration } = params;

	const rows = events
		.map((e) => {
			const statusClass = e.status === 'in' ? 'status-in' : 'status-out';
			const statusText = e.status === 'in' ? 'Clocked In' : 'Clocked Out';
			const clockOut = e.clock_out ? formatDateTime(e.clock_out) : '—';
			return `<tr>
	<td>${e.first_name} ${e.last_name}</td>
	<td>${e.email}</td>
	<td>${formatDateTime(e.clock_in)}</td>
	<td>${clockOut}</td>
	<td>${formatDuration(e.clock_in, e.clock_out)}</td>
	<td class="${statusClass}">${statusText}</td>
</tr>`;
		})
		.join('');

	return `<!DOCTYPE html>
<html>
<head>
<title>Attendance Report - ${companyName}</title>
<style>
body { font-family: Arial, sans-serif; margin: 20px; color: #333; }
h1 { color: #3D7A82; margin-bottom: 4px; }
.meta { color: #666; margin-bottom: 16px; font-size: 14px; }
table { width: 100%; border-collapse: collapse; font-size: 13px; }
th { background-color: #3D7A82; color: white; text-align: left; padding: 8px 12px; }
td { padding: 8px 12px; border-bottom: 1px solid #ddd; }
tr:nth-child(even) { background-color: #f8f9fa; }
.status-in { color: #16a34a; font-weight: bold; }
.status-out { color: #6b7280; }
.footer { margin-top: 20px; font-size: 11px; color: #999; }
@media print { body { margin: 0; } }
</style>
</head>
<body>
<h1>Attendance Report</h1>
<div class="meta">${companyName} — ${dateRange} — Generated ${new Date().toLocaleString('en-GB')}</div>
<table>
<thead><tr><th>Employee</th><th>Email</th><th>Clock In</th><th>Clock Out</th><th>Duration</th><th>Status</th></tr></thead>
<tbody>
${rows}
</tbody>
</table>
<div class="footer">LogSmart Attendance Report — ${events.length} record(s)</div>
</body>
</html>`;
}

export const PDF_STYLES = {
	report: `
		body { font-family: Arial, sans-serif; margin: 20px; }
		.header { border-bottom: 2px solid #333; margin-bottom: 20px; padding-bottom: 10px; }
		.entry { border: 1px solid #ddd; margin: 10px 0; padding: 15px; border-radius: 5px; page-break-inside: avoid; }
		.status { padding: 3px 8px; border-radius: 3px; color: white; font-size: 12px; }
		.submitted { background-color: #10B981; }
		.draft { background-color: #F59E0B; }
		.group-header { font-size: 18px; font-weight: bold; margin: 20px 0 10px 0; border-bottom: 1px solid #ccc; padding-bottom: 5px; }
		.entry-data { background-color: #f5f5f5; padding: 10px; margin: 5px 0; border-radius: 3px; }
		@media print { body { margin: 0; } .entry { page-break-inside: avoid; } }
	`,
	word: `
		body { 
			font-family: Aptos, Verdana, 'Segoe UI', Arial, sans-serif; 
			font-size: 11pt; 
			margin: 1in; 
			line-height: 1.4; 
			text-align: left;
		}
		h1 { font-size: 18pt; font-weight: bold; text-align: center; margin-bottom: 14pt; }
		h2 { font-size: 14pt; font-weight: bold; margin-top: 16pt; margin-bottom: 8pt; text-align: left; }
		p { margin: 4pt 0; line-height: 1.4; text-align: left; }
		.header { border-bottom: 2pt solid black; padding-bottom: 8pt; margin-bottom: 16pt; text-align: left; }
		.entry-box { 
			border: 1pt solid #999; 
			margin: 14pt 0; 
			padding: 12pt; 
			text-align: left;
			page-break-inside: avoid;
			display: block;
			orphans: 10;
			widows: 10;
		}
		.entry-title { font-size: 12pt; font-weight: bold; margin-bottom: 4pt; }
		.entry-id { font-size: 9pt; color: #666; margin-bottom: 4pt; }
		.status-badge { padding: 3pt 8pt; color: white; font-size: 10pt; margin-bottom: 8pt; display: inline-block; }
		.entry-data-box { background-color: #f5f5f5; padding: 10pt; margin: 8pt 0; border-left: 3pt solid #10B981; }
		.field-row { margin: 6pt 0; line-height: 1.5; }
		.field-label { font-weight: bold; color: #333; }
	`
};
