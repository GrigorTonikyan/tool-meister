import express from 'express';
import { createServer } from 'http';
import { WebSocketServer } from 'ws';
import { watch } from 'fs';
import { readdir, stat, writeFile } from 'fs/promises';
import { join, relative, resolve, parse, dirname } from 'path';

const app = express();
const server = createServer(app);
const wss = new WebSocketServer({ server });

let projectRoot = resolve('../');
let watcher = null;

// Store recent projects
const RECENT_PROJECTS_FILE = '.recent_projects';
let recentProjects = new Set();

try {
    const recentProjectsData = await readdir(RECENT_PROJECTS_FILE);
    recentProjects = new Set(recentProjectsData);
} catch (err) {
    console.log('No recent projects file found');
}

// CORS middleware
app.use((req, res, next) => {
    res.header('Access-Control-Allow-Origin', '*');
    res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.header('Access-Control-Allow-Headers', 'Content-Type');
    if (req.method === 'OPTIONS') {
        return res.sendStatus(200);
    }
    next();
});

// Middleware
app.use(express.json());

// API Routes - Add these before static file serving
app.get('/api/directories', async (req, res) => {
    try {
        const path = req.query.path || '/';
        const resolvedPath = resolve(path);
        
        console.log('Loading directories for path:', resolvedPath);
        
        const items = await readdir(resolvedPath);
        const directories = [];
        
        for (const item of items) {
            try {
                const fullPath = join(resolvedPath, item);
                const stats = await stat(fullPath);
                
                if (stats.isDirectory()) {
                    directories.push({
                        name: item,
                        path: fullPath,
                        isParent: false
                    });
                }
            } catch (err) {
                console.error(`Error processing ${item}:`, err);
            }
        }
        
        // Add parent directory except for root
        if (resolvedPath !== '/') {
            directories.unshift({
                name: '..',
                path: dirname(resolvedPath),
                isParent: true
            });
        }
        
        const response = {
            current: resolvedPath,
            items: directories
        };
        
        console.log('Sending directory response:', response);
        res.json(response);
    } catch (err) {
        console.error('Error listing directories:', err);
        res.status(500).json({ error: err.message });
    }
});

app.get('/api/recent-projects', (req, res) => {
    console.log('Getting recent projects:', Array.from(recentProjects));
    res.json(Array.from(recentProjects));
});

app.post('/api/set-project', async (req, res) => {
    try {
        const { path } = req.body;
        console.log('Setting project path:', path);
        
        const resolvedPath = resolve(path);
        
        // Verify it's a valid directory
        const stats = await stat(resolvedPath);
        if (!stats.isDirectory()) {
            throw new Error('Not a directory');
        }
        
        // Update project root and restart watcher
        projectRoot = resolvedPath;
        
        // Update recent projects
        recentProjects.add(resolvedPath);
        if (recentProjects.size > 10) {
            recentProjects.delete(Array.from(recentProjects)[0]);
        }
        
        // Save recent projects
        try {
            await writeFile(RECENT_PROJECTS_FILE, Array.from(recentProjects).join('\n'));
        } catch (err) {
            console.error('Error saving recent projects:', err);
        }
        
        // Restart watcher
        if (watcher) {
            watcher.close();
        }
        setupWatcher();
        
        // Broadcast new structure to all clients
        const structure = await getDirectoryStructure(projectRoot);
        broadcastToClients({
            type: 'structure',
            content: {
                name: parse(projectRoot).base,
                path: '/',
                type: 'directory',
                children: structure
            }
        });
        
        res.json({ success: true });
    } catch (err) {
        console.error('Error setting project:', err);
        res.status(500).json({ error: err.message });
    }
});

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// Serve static files - This should come after API routes
app.use(express.static('.'));

// Serve index.html for all other routes
app.get('*', (req, res) => {
    res.sendFile(resolve(__dirname, 'index.html'));
});

async function getDirectoryStructure(dir) {
    try {
        const items = await readdir(dir);
        const structure = [];

        for (const item of items) {
            try {
                const path = join(dir, item);
                const stats = await stat(path);
                const relativePath = relative(projectRoot, path);

                if (item === 'node_modules' || item === '.git') continue;

                const node = {
                    name: item,
                    path: relativePath,
                    type: stats.isDirectory() ? 'directory' : 'file',
                    size: stats.isFile() ? stats.size : null
                };

                if (stats.isDirectory()) {
                    node.children = await getDirectoryStructure(path);
                }

                structure.push(node);
            } catch (err) {
                console.error(`Error processing ${item}:`, err);
            }
        }

        return structure;
    } catch (err) {
        console.error(`Error reading directory ${dir}:`, err);
        return [];
    }
}

function broadcastToClients(message) {
    const clients = Array.from(wss.clients);
    if (clients.length === 0) return;

    const messageStr = JSON.stringify(message);
    clients.forEach(client => {
        if (client.readyState === 1) { // OPEN
            client.send(messageStr);
        }
    });
}

function setupWatcher() {
    watcher = watch(projectRoot, { recursive: true }, async (eventType, filename) => {
        if (!filename) return;
        
        // Ignore node_modules and .git
        if (filename.includes('node_modules') || filename.includes('.git')) return;
        
        console.log(`File ${filename} changed`);
        
        try {
            // Send file change notification
            broadcastToClients({
                type: 'fileChange',
                file: filename
            });
            
            // Send updated structure
            const structure = await getDirectoryStructure(projectRoot);
            broadcastToClients({
                type: 'structure',
                content: {
                    name: parse(projectRoot).base,
                    path: '/',
                    type: 'directory',
                    children: structure
                }
            });
        } catch (err) {
            console.error('Error broadcasting changes:', err);
        }
    });

    watcher.on('error', (err) => {
        console.error('File watcher error:', err);
    });
}

// Handle WebSocket connections
wss.on('connection', async (ws) => {
    console.log('Client connected');

    // Send initial structure
    try {
        const structure = await getDirectoryStructure(projectRoot);
        ws.send(JSON.stringify({
            type: 'structure',
            content: {
                name: parse(projectRoot).base,
                path: '/',
                type: 'directory',
                children: structure
            }
        }));
    } catch (err) {
        console.error('Error sending initial structure:', err);
        ws.send(JSON.stringify({
            type: 'error',
            message: 'Failed to load project structure'
        }));
    }

    ws.on('message', async (message) => {
        try {
            const data = JSON.parse(message);
            if (data.type === 'getStructure') {
                const structure = await getDirectoryStructure(projectRoot);
                ws.send(JSON.stringify({
                    type: 'structure',
                    content: {
                        name: parse(projectRoot).base,
                        path: '/',
                        type: 'directory',
                        children: structure
                    }
                }));
            }
        } catch (err) {
            console.error('Error handling message:', err);
            ws.send(JSON.stringify({
                type: 'error',
                message: 'Failed to process request'
            }));
        }
    });

    ws.on('error', (err) => {
        console.error('WebSocket error:', err);
    });
});

// Initial watcher setup
setupWatcher();

// Start the server
const port = 3001;
server.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
    console.log(`WebSocket server running at ws://localhost:${port}`);
    console.log(`Watching directory: ${projectRoot}`);
});
