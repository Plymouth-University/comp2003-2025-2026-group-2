import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import { mkdtempSync, rmSync, writeFileSync } from 'fs';
import os from 'os';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BACKEND_PORT = process.env.BACKEND_PORT || 6767;
const BACKEND_URL = `http://localhost:${BACKEND_PORT}`;
const FRONTEND_PORT = process.env.FRONTEND_PORT || 5173;
const FRONTEND_URL = `http://localhost:${FRONTEND_PORT}`;
const MAILHOG_API_URL = process.env.MAILHOG_API_URL || 'http://localhost:8025/api';
const PID_FILE = path.join(os.tmpdir(), 'logsmart-test-pids.json');
let backendProcess: any = null;
let frontendProcess: any = null;
let tempDir: string | null = null;

async function waitForServer(url: string, timeout = 1200000): Promise<void> {
	const startTime = Date.now();
	while (Date.now() - startTime < timeout) {
		try {
			const response = await fetch(url);
			if (response.ok || response.status === 404) {
				console.log(`✓ Server is ready at ${url}`);
				return;
			}
		} catch (e) {}
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	throw new Error(`Server did not start within ${timeout}ms at ${url}`);
}

async function clearMailhogEmails(): Promise<void> {
	try {
		const response = await fetch(`${MAILHOG_API_URL}/v1/messages`, { method: 'DELETE' });
		if (response.ok) {
			console.log('✓ Mailhog emails cleared');
		}
	} catch (error) {
		console.warn('⚠ Could not clear Mailhog emails (Mailhog may not be running):', error);
	}
}

async function globalSetup() {
	tempDir = mkdtempSync(path.join(os.tmpdir(), 'logsmart-test-'));

	console.log('Starting backend server...');

	const isWindows = process.platform === 'win32';
	const backendSourceDir = path.resolve(__dirname, '../../../back-end');

	const backendEnv = {
		...process.env,
		DISABLE_RATE_LIMIT: '1',
		SMTP_SERVER: '127.0.0.1:1025',
		SMTP_USERNAME: '',
		SMTP_PASSWORD: '',
		SMTP_FROM_EMAIL: 'noreply@logsmart.app',
		SMTP_FROM_NAME: 'LogSmart',
		DATABASE_URL: 'sqlite:auth.db',
		MONGODB_URI: 'mongodb://root:rootpassword@127.0.0.1'
	};

	if (isWindows) {
		backendProcess = spawn('cargo', ['run'], {
			cwd: backendSourceDir,
			shell: true,
			env: backendEnv
		});
	} else {
		backendProcess = spawn('nix', ['run', `${backendSourceDir}`], {
			cwd: tempDir,
			shell: true,
			env: backendEnv
		});
	}

	// backendProcess.stdout?.on('data', (data: Buffer) => {
	// 	console.log(`[Backend] ${data}`);
	// });

	// backendProcess.stderr?.on('data', (data: Buffer) => {
	// 	console.log(`[Backend Error] ${data}`);
	// });

	backendProcess.on('error', (error: Error) => {
		console.error(`Failed to start backend: ${error.message}`);
	});

	await waitForServer(BACKEND_URL);
	process.env.BACKEND_URL = BACKEND_URL;
	process.env.PUBLIC_API_URL = BACKEND_URL;

	let pidData = {
		backendPid: backendProcess?.pid,
		frontendPid: null,
		tempDir
	};
	writeFileSync(PID_FILE, JSON.stringify(pidData, null, 2));

	console.log('Starting frontend dev server...');
	``;
	const frontendDir = path.resolve(__dirname, '..');
	frontendProcess = spawn('bun', ['run', 'dev'], {
		cwd: frontendDir,
		shell: true,
		env: { ...process.env }
	});

	// frontendProcess.stdout?.on('data', (data: Buffer) => {
	// 	console.log(`[Frontend] ${data}`);
	// });

	// frontendProcess.stderr?.on('data', (data: Buffer) => {
	// 	console.log(`[Frontend Error] ${data}`);
	// });

	frontendProcess.on('error', (error: Error) => {
		console.error(`Failed to start frontend: ${error.message}`);
	});

	await waitForServer(FRONTEND_URL);
	process.env.FRONTEND_URL = FRONTEND_URL;

	await clearMailhogEmails();

	pidData = {
		backendPid: backendProcess?.pid,
		frontendPid: frontendProcess?.pid,
		tempDir
	};
	writeFileSync(PID_FILE, JSON.stringify(pidData, null, 2));

	const cleanupProcesses = () => {
		if (backendProcess?.pid) {
			try {
				process.kill(backendProcess.pid, 'SIGTERM');
			} catch (e) {}
		}
		if (frontendProcess?.pid) {
			try {
				process.kill(frontendProcess.pid, 'SIGTERM');
			} catch (e) {}
		}
		if (tempDir) {
			try {
				rmSync(tempDir, { recursive: true, force: true });
			} catch (e) {}
		}
	};

	process.on('exit', cleanupProcesses);
	process.on('SIGINT', () => {
		cleanupProcesses();
		process.exit(0);
	});
	process.on('SIGTERM', () => {
		cleanupProcesses();
		process.exit(0);
	});
}

export default globalSetup;
