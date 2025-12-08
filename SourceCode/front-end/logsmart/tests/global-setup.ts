import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import { mkdtempSync, rmSync, copyFile } from 'fs';
import os from 'os';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BACKEND_PORT = process.env.BACKEND_PORT || 6767;
const BACKEND_URL = `http://localhost:${BACKEND_PORT}`;
const FRONTEND_PORT = process.env.FRONTEND_PORT || 5173;
const FRONTEND_URL = `http://localhost:${FRONTEND_PORT}`;
let backendProcess: any = null;
let frontendProcess: any = null;
let tempDir: string | null = null;

async function waitForServer(url: string, timeout = 30000): Promise<void> {
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

async function globalSetup() {
	tempDir = mkdtempSync(path.join(os.tmpdir(), 'logsmart-test-'));

	console.log('Starting backend server...');

  const isWindows = process.platform === 'win32';
  const backendSourceDir = path.resolve(__dirname, '../../../back-end');

  await copyFile(path.join(backendSourceDir, '.env'), path.join(tempDir, '.env'), (err) => {
    if (err) throw err;
  });

  if (isWindows) {
    backendProcess = spawn('cargo', ['run'], {
      cwd: backendSourceDir,
      shell: true
    });
  } else {
    backendProcess = spawn('nix', ['run', `${backendSourceDir}`], {
      cwd: tempDir,
      shell: true
    });
  }

	// backendProcess.stderr?.on('data', (data: Buffer) => {
	// 	console.log(`[Backend Error] ${data}`);
	// });

	backendProcess.on('error', (error: Error) => {
		console.error(`Failed to start backend: ${error.message}`);
	});

	await waitForServer(BACKEND_URL);
	process.env.BACKEND_URL = BACKEND_URL;
	process.env.PUBLIC_API_URL = BACKEND_URL;

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

	process.on('exit', () => {
		if (backendProcess) backendProcess.kill();
		if (frontendProcess) frontendProcess.kill();
		if (tempDir) {
			console.log('Cleaning up temporary directory...');
			rmSync(tempDir, { recursive: true, force: true });
		}
	});
}

export default globalSetup;
