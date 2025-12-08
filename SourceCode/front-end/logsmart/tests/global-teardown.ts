let backendProcess: any = null;
let frontendProcess: any = null;

async function globalTeardown() {
	console.log('Stopping backend server...');

	if (backendProcess && backendProcess.pid) {
		console.log(`Killing process ${backendProcess.pid}`);
		try {
			process.kill(-backendProcess.pid);
			await new Promise((resolve) => setTimeout(resolve, 1000));
		} catch (error) {
			console.error('Error killing backend process:', error);
		}
	}
	if (frontendProcess && frontendProcess.pid) {
		console.log(`Killing process ${frontendProcess.pid}`);
		try {
			process.kill(-frontendProcess.pid);
			await new Promise((resolve) => setTimeout(resolve, 1000));
		} catch (error) {
			console.error('Error killing frontend process:', error);
		}
	}

	console.log('Backend server stopped');
}

export default globalTeardown;
