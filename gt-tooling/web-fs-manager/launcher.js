import { spawn } from 'child_process';
import fs from 'fs/promises';
import { dirname, join } from 'path';
import treeKill from 'tree-kill';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Process ID file
const PID_FILE = join(__dirname, '.process-ids');

// Store running processes
const processes = new Map();

async function savePids() {
    const pids = Array.from(processes.entries()).map(([name, proc]) => `${name}:${proc.pid}`);
    await fs.writeFile(PID_FILE, pids.join('\n'));
}

async function loadPids() {
    try {
        const content = await fs.readFile(PID_FILE, 'utf-8');
        return new Map(
            content.split('\n')
                .filter(Boolean)
                .map(line => {
                    const [name, pid] = line.split(':');
                    return [name, parseInt(pid)];
                })
        );
    } catch (err) {
        return new Map();
    }
}

function startProcess(name, command, args = [], options = {}) {
    console.log(`Starting ${name}...`);

    const proc = spawn(command, args, {
        ...options,
        stdio: 'inherit',
        shell: true
    });

    processes.set(name, proc);

    proc.on('error', (err) => {
        console.error(`${name} error:`, err);
    });

    proc.on('exit', (code) => {
        if (code !== 0) {
            console.error(`${name} exited with code ${code}`);
        }
        processes.delete(name);
        savePids();
    });

    return proc;
}

async function stopAll() {
    console.log('Stopping all processes...');

    // Load PIDs from file in case the script was restarted
    const savedPids = await loadPids();
    for (const [name, pid] of savedPids) {
        console.log(`Stopping ${name} (PID: ${pid})...`);
        try {
            treeKill(pid);
        } catch (err) {
            console.error(`Error stopping ${name}:`, err);
        }
    }

    // Stop currently running processes
    for (const [name, proc] of processes) {
        console.log(`Stopping ${name} (PID: ${proc.pid})...`);
        try {
            treeKill(proc.pid);
        } catch (err) {
            console.error(`Error stopping ${name}:`, err);
        }
    }

    // Clear PID file
    try {
        await fs.unlink(PID_FILE);
    } catch (err) {
        // Ignore if file doesn't exist
    }

    processes.clear();
}

async function start() {
    // First stop any running processes
    await stopAll();

    console.log('Starting all processes...');

    // Start the WebSocket server
    const server = startProcess(
        'server',
        'node',
        ['server.js'],
        { cwd: __dirname }
    );

    // Wait a bit for the server to start
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Start Vite development server
    const vite = startProcess(
        'vite',
        'pnpm',
        ['run', 'dev'],
        { cwd: __dirname }
    );

    // Save process IDs
    await savePids();

    // Handle process termination
    process.on('SIGINT', handleShutdown);
    process.on('SIGTERM', handleShutdown);
    process.on('uncaughtException', handleShutdown);
    process.on('unhandledRejection', handleShutdown);
}

async function handleShutdown(err) {
    if (err) console.error('Error:', err);
    console.log('\nShutting down...');
    await stopAll();
    process.exit(0);
}

// Check command line arguments
const command = process.argv[2];

if (command === 'stop') {
    await stopAll();
    process.exit(0);
} else {
    await start();
}
