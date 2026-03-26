import * as d3 from 'd3';

let root = null;
let svg = null;
let treeLayout = null;
const margin = { top: 40, right: 90, bottom: 50, left: 90 };
const width = 800;
const height = 600;

// WebSocket connection management
let ws = null;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;
const reconnectDelay = 1000;

// Project selection
let currentPath = '/';

// Expose functions to window for HTML onclick handlers
window.showProjectSelector = showProjectSelector;
window.hideProjectSelector = hideProjectSelector;
window.selectDirectory = selectDirectory;
window.navigateTo = navigateTo;
window.selectProject = selectProject;
window.selectCurrentDirectory = selectCurrentDirectory;

async function showProjectSelector() {
    const modal = document.getElementById('project-modal');
    modal.classList.add('visible');
    await loadDirectories(currentPath);
    await loadRecentProjects();
    updateSelectButton();
}

function hideProjectSelector() {
    const modal = document.getElementById('project-modal');
    modal.classList.remove('visible');
}

async function loadDirectories(path) {
    try {
        const response = await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
        const data = await response.json();

        console.log('~~~~ data:', data);

        currentPath = data.current;
        updateBreadcrumb(currentPath);

        const directoryList = document.getElementById('directory-list');
        directoryList.innerHTML = '';

        data.items.forEach(item => {
            const li = document.createElement('li');
            li.className = 'directory-item' + (item.isParent ? ' parent' : '');
            li.innerHTML = `
                <i>${item.isParent ? '📁' : '📂'}</i>
                ${item.name}
            `;
            li.onclick = () => selectDirectory(item.path);
            directoryList.appendChild(li);
        });
    } catch (err) {
        console.error('Error loading directories:', err);
        showStatus('Error loading directories', 'error');
    }
}

async function loadRecentProjects() {
    try {
        const response = await fetch('/api/recent-projects');
        const projects = await response.json();

        const recentList = document.getElementById('recent-projects-list');
        recentList.innerHTML = '';

        projects.forEach(project => {
            const li = document.createElement('li');
            li.className = 'directory-item';
            li.innerHTML = `
                <i>📂</i>
                ${project}
            `;
            li.onclick = () => selectProject(project);
            recentList.appendChild(li);
        });
    } catch (err) {
        console.error('Error loading recent projects:', err);
        showStatus('Error loading recent projects', 'error');
    }
}

function updateBreadcrumb(path) {
    const breadcrumb = document.getElementById('path-breadcrumb');
    breadcrumb.innerHTML = '';

    const parts = path.split('/').filter(Boolean);
    let currentPath = '';

    // Add root
    const rootItem = document.createElement('span');
    rootItem.className = 'breadcrumb-item';
    rootItem.textContent = '/';
    rootItem.onclick = () => navigateTo('/');
    breadcrumb.appendChild(rootItem);

    parts.forEach((part, index) => {
        if (index > 0) {
            const separator = document.createElement('span');
            separator.className = 'breadcrumb-separator';
            separator.textContent = '/';
            breadcrumb.appendChild(separator);
        }

        currentPath += '/' + part;
        const item = document.createElement('span');
        item.className = 'breadcrumb-item';
        item.textContent = part;
        item.onclick = () => navigateTo(currentPath);
        breadcrumb.appendChild(item);
    });
}

async function selectDirectory(path) {
    await loadDirectories(path);
}

async function navigateTo(path) {
    await loadDirectories(path);
}

async function selectProject(path) {
    try {
        const response = await fetch('/api/set-project', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ path })
        });

        if (!response.ok) {
            throw new Error('Failed to set project');
        }

        document.getElementById('current-project').textContent = path;
        hideProjectSelector();
        showStatus('Project changed: ' + path);
    } catch (err) {
        console.error('Error selecting project:', err);
        showStatus('Error selecting project', 'error');
    }
}

function updateSelectButton() {
    const selectBtn = document.getElementById('select-dir-btn');
    selectBtn.textContent = `Select "${currentPath}"`;
}

async function selectCurrentDirectory() {
    await selectProject(currentPath);
}

function connectWebSocket() {
    ws = new WebSocket('ws://localhost:3001/ws');

    ws.onopen = () => {
        console.log('Connected to WebSocket server');
        showStatus('Connected to server');
        reconnectAttempts = 0;
        requestStructure();
    };

    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            if (data.type === 'structure') {
                updateTree(data.content);
                showStatus('Structure Updated');
            } else if (data.type === 'fileChange') {
                showStatus(`File Changed: ${data.file}`);
                highlightNode(data.file);
            } else if (data.type === 'error') {
                showStatus(`Error: ${data.message}`, 'error');
            }
        } catch (err) {
            console.error('Error processing message:', err);
            showStatus('Error processing server message', 'error');
        }
    };

    ws.onclose = () => {
        console.log('WebSocket connection closed');
        if (reconnectAttempts < maxReconnectAttempts) {
            reconnectAttempts++;
            showStatus(`Connection lost. Reconnecting (${reconnectAttempts}/${maxReconnectAttempts})...`, 'warning');
            setTimeout(connectWebSocket, reconnectDelay);
        } else {
            showStatus('Connection lost. Please refresh the page.', 'error');
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        showStatus('Connection error', 'error');
    };
}

function requestStructure() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'getStructure' }));
    }
}

function showStatus(message, type = 'info') {
    const status = document.getElementById('status');
    status.textContent = message;
    status.className = 'status visible';

    // Add color based on message type
    if (type === 'error') {
        status.style.backgroundColor = '#d32f2f';
    } else if (type === 'warning') {
        status.style.backgroundColor = '#f57c00';
    } else {
        status.style.backgroundColor = '#0078d4';
    }

    setTimeout(() => status.classList.remove('visible'), 3000);
}

function updateTree(data) {
    try {
        root = d3.hierarchy(data);

        const treeLayout = d3.tree()
            .size([height - margin.top - margin.bottom, width - margin.left - margin.right]);

        treeLayout(root);

        if (!svg) {
            svg = d3.select('#tree-container')
                .append('svg')
                .attr('width', '100%')
                .attr('height', '100%')
                .attr('viewBox', `0 0 ${width} ${height}`)
                .append('g')
                .attr('transform', `translate(${margin.left},${margin.top})`);
        }

        // Clear existing nodes and links
        svg.selectAll('.link').remove();
        svg.selectAll('.node').remove();

        // Add links
        const links = svg.selectAll('.link')
            .data(root.links())
            .enter()
            .append('path')
            .attr('class', 'link')
            .attr('d', d3.linkHorizontal()
                .x(d => d.y)
                .y(d => d.x));

        // Add nodes
        const nodes = svg.selectAll('.node')
            .data(root.descendants())
            .enter()
            .append('g')
            .attr('class', 'node')
            .attr('transform', d => `translate(${d.y},${d.x})`);

        nodes.append('circle')
            .attr('r', 6);

        nodes.append('text')
            .attr('dy', '.31em')
            .attr('x', d => d.children ? -8 : 8)
            .style('text-anchor', d => d.children ? 'end' : 'start')
            .text(d => d.data.name);

        // Add click handler
        nodes.on('click', (event, d) => {
            updateFileInfo(d.data);
            nodes.classed('selected', false);
            d3.select(event.currentTarget).classed('selected', true);
        });
    } catch (err) {
        console.error('Error updating tree:', err);
        showStatus('Error updating visualization', 'error');
    }
}

function updateFileInfo(data) {
    const fileDetails = document.getElementById('file-details');
    fileDetails.innerHTML = `
        <div class="file-info">
            <h3>${data.name}</h3>
            <p>Type: ${data.type || 'N/A'}</p>
            <p>Size: ${formatSize(data.size) || 'N/A'}</p>
            ${data.children ? `<p>Children: ${data.children.length}</p>` : ''}
            <p>Path: ${data.path || 'N/A'}</p>
        </div>
    `;
}

function formatSize(bytes) {
    if (!bytes) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function highlightNode(path) {
    if (!root) return;

    const nodes = svg.selectAll('.node');
    nodes.classed('selected', d => d.data.path === path);
}

// Initialize WebSocket connection
connectWebSocket();
