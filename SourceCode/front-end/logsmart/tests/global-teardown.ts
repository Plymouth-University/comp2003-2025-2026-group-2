import { readFileSync, rmSync } from 'fs';
import path from 'path';
import os from 'os';

const PID_FILE = path.join(os.tmpdir(), 'logsmart-test-pids.json');

async function globalTeardown() {
	console.log('Stopping servers...');

	try {
		const pidData = JSON.parse(readFileSync(PID_FILE, 'utf-8'));
		const { backendPid, frontendPid, tempDir } = pidData;

		if (backendPid) {
			console.log(`Killing backend process ${backendPid}`);
			try {
				process.kill(backendPid, 'SIGTERM');
				await new Promise((resolve) => setTimeout(resolve, 500));
			} catch (error: any) {
				if (error.code !== 'ESRCH') {
					console.log('Error killing backend process:', error.message);
				}
			}
		}

		if (frontendPid) {
			console.log(`Killing frontend process ${frontendPid}`);
			try {
				process.kill(frontendPid, 'SIGTERM');
				await new Promise((resolve) => setTimeout(resolve, 500));
			} catch (error: any) {
				if (error.code !== 'ESRCH') {
					console.log('Error killing frontend process:', error.message);
				}
			}
		}

		if (tempDir) {
			try {
				rmSync(tempDir, { recursive: true, force: true });
				console.log('Temporary directory cleaned up');
			} catch (error) {
				console.log('Temporary directory cleanup failed or already removed');
			}
		}

		try {
			rmSync(PID_FILE);
		} catch (e) {}
	} catch (error) {
		console.log('PID file not found or already cleaned up');
	}

	console.log('Servers stopped');
}

export default globalTeardown;
