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
const GOOGLE_ISSUER_URL = 'http://localhost:8080/google';
const MAILHOG_API_URL = process.env.MAILHOG_API_URL || 'http://localhost:8025/api';
const PID_FILE = path.join(os.tmpdir(), 'logsmart-test-pids.json');
const backendProcess: any = null;
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

async function checkMockOAuth(timeout = 30000): Promise<void> {
	const startTime = Date.now();
	while (Date.now() - startTime < timeout) {
		try {
			const response = await fetch(`${GOOGLE_ISSUER_URL}/.well-known/openid-configuration`);
			if (response.ok) {
				console.log('✓ Mock OAuth server is running');
				return;
			}
			console.log(`Mock OAuth check: HTTP ${response.status}`);
		} catch (e: any) {
			console.log(`Mock OAuth check error: ${e.message}`);
		}
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	throw new Error(
		'Mock OAuth server is not running at http://localhost:8080. Please start it with docker-compose.'
	);
}

async function dropAllTables(timeout = 5000): Promise<void> {
	try {
		const { Client } = await import('pg');
		const client = new Client({
			host: '127.0.0.1',
			port: 5432,
			user: 'admin',
			password: 'adminpassword',
			database: 'logsmartdb'
		});

		await client.connect();
		await client.query('DROP SCHEMA public CASCADE');
		await client.query('CREATE SCHEMA public');
		await client.query('GRANT ALL ON SCHEMA public TO public');
		await client.end();
		console.log('✓ All database tables dropped');
	} catch (error) {
		console.warn('⚠ Could not drop database tables:', error);
	}
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

// Backend is expected to be started via docker-compose by the user.
// The setup will wait for the backend health endpoint to become available.

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

	console.log('Checking Mock OAuth server...');
	await checkMockOAuth();

	console.log('Checking PostgreSQL...');
	await checkPostgres();

	// console.log('Dropping all database tables...');
	// await dropAllTables();

	console.log('Checking backend service (ensure containers are started)...');
	try {
		await waitForServer(BACKEND_URL, 120000);
		console.log(`✓ Backend reachable at ${BACKEND_URL}`);
	} catch (err) {
		console.error('Backend did not become ready:', err);
		throw new Error(
			'Backend service is not up. Please ensure containers are started (docker compose -d)'
		);
	}

	console.log('Starting frontend server (local)...');
	try {
		await startFrontendProcess();
	} catch (error) {
		console.error('Failed to start frontend:', error);
		throw error;
	}

	await clearMailhogEmails();

	const pidData = {
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
