import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import { mkdtempSync, rmSync, writeFileSync } from 'fs';
import os from 'os';
import { createConnection } from 'net';

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

async function checkMongoDB(timeout = 5000): Promise<void> {
	const startTime = Date.now();
	while (Date.now() - startTime < timeout) {
		try {
			await new Promise<void>((resolve, reject) => {
				const socket = createConnection({ host: '127.0.0.1', port: 27017 });
				socket.on('connect', () => {
					socket.destroy();
					resolve();
				});
				socket.on('error', (err) => {
					reject(err);
				});
				setTimeout(() => {
					socket.destroy();
					reject(new Error('Connection timeout'));
				}, 1000);
			});
			console.log('✓ MongoDB is running');
			return;
		} catch (e) {}
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	throw new Error(
		'MongoDB is not running on 127.0.0.1:27017. Please start MongoDB before running tests.'
	);
}

async function checkMailhog(timeout = 5000): Promise<void> {
	const startTime = Date.now();
	while (Date.now() - startTime < timeout) {
		try {
			const response = await fetch(`${MAILHOG_API_URL}/v1/messages`);
			if (response.ok || response.status === 404 || response.status === 405) {
				console.log('✓ MailHog is running');
				return;
			}
		} catch (e) {}
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	throw new Error(
		'MailHog is not running at http://localhost:8025. Please start MailHog before running tests.'
	);
}

async function checkPostgres(timeout = 5000): Promise<void> {
	const startTime = Date.now();
	while (Date.now() - startTime < timeout) {
		try {
			await new Promise<void>((resolve, reject) => {
				const socket = createConnection({ host: '127.0.0.1', port: 5432 });
				socket.on('connect', () => {
					socket.destroy();
					resolve();
				});
				socket.on('error', (err) => {
					reject(err);
				});
				setTimeout(() => {
					socket.destroy();
					reject(new Error('Connection timeout'));
				}, 1000);
			});
			console.log('✓ PostgreSQL is running');
			return;
		} catch (e) {}
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	throw new Error(
		'PostgreSQL is not running on 127.0.0.1:5432. Please start PostgreSQL before running tests.'
	);
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

async function startBackendProcess(): Promise<void> {
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
		POSTGRES_HOST: 'localhost',
		POSTGRES_PORT: '5432',
		POSTGRES_DB: 'logsmartdb',
		POSTGRES_USER: 'admin',
		POSTGRES_PASSWORD: 'adminpassword',
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
			cwd: tempDir!,
			shell: true,
			env: backendEnv
		});
	}

	backendProcess.stdout?.on('data', (data: Buffer) => {
		console.log(`[Backend] ${data.toString().trim()}`);
	});

	backendProcess.stderr?.on('data', (data: Buffer) => {
		console.log(`[Backend Error] ${data.toString().trim()}`);
	});

	backendProcess.on('error', (error: Error) => {
		console.error(`Failed to start backend: ${error.message}`);
	});

	await waitForServer(BACKEND_URL);
	process.env.BACKEND_URL = BACKEND_URL;
	process.env.PUBLIC_API_URL = BACKEND_URL;
}

async function startFrontendProcess(): Promise<void> {
	const frontendDir = path.resolve(__dirname, '..');
	frontendProcess = spawn('bun', ['run', 'dev'], {
		cwd: frontendDir,
		shell: true,
		env: { ...process.env, PUBLIC_API_URL: BACKEND_URL }
	});

	// frontendProcess.stdout?.on('data', (data: Buffer) => {
	// 	console.log(`[Frontend] ${data.toString().trim()}`);
	// });

	// frontendProcess.stderr?.on('data', (data: Buffer) => {
	// 	console.log(`[Frontend Error] ${data.toString().trim()}`);
	// });

	frontendProcess.on('error', (error: Error) => {
		console.error(`Failed to start frontend: ${error.message}`);
	});

	await waitForServer(FRONTEND_URL);
	process.env.FRONTEND_URL = FRONTEND_URL;
}

async function globalSetup() {
	tempDir = mkdtempSync(path.join(os.tmpdir(), 'logsmart-test-'));

	console.log('Checking MongoDB...');
	await checkMongoDB();

	console.log('Checking MailHog...');
	await checkMailhog();

	console.log('Checking PostgreSQL...');
	await checkPostgres();

	console.log('Starting backend and frontend servers...');

	try {
		await Promise.all([startBackendProcess(), startFrontendProcess()]);
	} catch (error) {
		console.error('Failed to start servers:', error);
		throw error;
	}

	await clearMailhogEmails();

	let pidData = {
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
