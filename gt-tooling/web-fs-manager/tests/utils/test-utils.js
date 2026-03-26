/**
 * @typedef {Object} DirectoryItem
 * @property {string} name - The name of the directory
 * @property {string} path - The absolute path of the directory
 * @property {boolean} isParent - Whether this is a parent directory
 * @property {boolean} [isSymlink] - Whether this is a symbolic link
 * @property {string} [symlink] - The target of the symbolic link
 */

/**
 * @typedef {Object} DirectoryResponse
 * @property {string} current - The current directory path
 * @property {DirectoryItem[]} items - List of directory items
 */

/**
 * Creates a mock directory response for testing
 * @param {string} currentPath - The current directory path
 * @param {string[]} dirNames - List of directory names to include
 * @returns {DirectoryResponse} Mock directory response
 */
export function createMockDirectoryResponse(currentPath, dirNames) {
    return {
        current: currentPath,
        items: dirNames.map(name => ({
            name,
            path: `${currentPath}/${name}`,
            isParent: false
        }))
    };
}

/**
 * Creates a mock DOM environment for directory selection tests
 * @returns {void}
 */
export function setupDirectorySelectionDOM() {
    document.body.innerHTML = `
        <div id="project-modal" class="modal">
            <div class="modal-content">
                <div class="modal-header">
                    <h2>Select Project Directory</h2>
                    <button class="modal-close">&times;</button>
                </div>
                <div class="breadcrumb" id="path-breadcrumb"></div>
                <div class="directory-actions">
                    <button class="btn" id="select-dir-btn">
                        Select Current Directory
                    </button>
                </div>
                <ul class="directory-list" id="directory-list"></ul>
                <div class="recent-projects">
                    <h3>Recent Projects</h3>
                    <ul class="directory-list" id="recent-projects-list"></ul>
                </div>
            </div>
        </div>
        <div id="status" class="status"></div>
        <div id="current-project">No project selected</div>
        <div id="tree-container"></div>
        <div id="file-info"></div>
        <div id="update-notifications"></div>
    `;
}

/**
 * Simulates a delay to mimic real-world async operations
 * @param {number} ms - Milliseconds to delay
 * @returns {Promise<void>}
 */
export function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Creates a mock file system structure for testing
 * @param {string} basePath - Base path for the file system
 * @returns {Object} Mock file system structure
 */
export function createMockFileSystem(basePath) {
    return {
        path: basePath,
        children: [
            {
                path: `${basePath}/src`,
                children: [
                    { path: `${basePath}/src/main.js`, size: 1024 },
                    { path: `${basePath}/src/utils.js`, size: 512 }
                ]
            },
            {
                path: `${basePath}/tests`,
                children: [
                    { path: `${basePath}/tests/main.test.js`, size: 768 }
                ]
            }
        ]
    };
}

/**
 * Measures the performance of a function
 * @template T
 * @param {() => Promise<T>} fn - Function to measure
 * @param {string} label - Label for the measurement
 * @returns {Promise<number>} Execution time in milliseconds
 */
export async function measurePerformance(fn, label) {
    const start = performance.now();
    await fn();
    const end = performance.now();
    const duration = end - start;
    console.log(`${label}: ${duration}ms`);
    return duration;
}

/**
 * Updates the DOM with directory content
 * @param {string} path - Current path
 * @param {DirectoryItem[]} items - Directory items
 */
export function updateDirectoryList(path, items) {
    const directoryList = document.getElementById('directory-list');
    const breadcrumb = document.getElementById('path-breadcrumb');
    const selectBtn = document.getElementById('select-dir-btn');

    // Update directory list
    directoryList.innerHTML = items.map(item => `
        <li class="directory-item${item.isParent ? ' parent' : ''}"
            data-path="${item.path}"
            ${item.isSymlink ? `data-symlink="${item.symlink}"` : ''}>
            <i>${item.isParent ? '📁' : item.isSymlink ? '🔗' : '📂'}</i>
            <span>${item.name}</span>
        </li>
    `).join('');

    // Update breadcrumb
    const parts = path.split('/').filter(Boolean);
    breadcrumb.innerHTML = parts.map((part, index) => {
        const currentPath = '/' + parts.slice(0, index + 1).join('/');
        return `<span class="breadcrumb-item" data-path="${currentPath}">${part}</span>`;
    }).join('/');
    breadcrumb.setAttribute('data-path', path);

    // Update select button
    selectBtn.textContent = `Select "${path}"`;
}

/**
 * Shows status message
 * @param {string} message - Status message
 * @param {'error' | 'success' | 'info'} [type='info'] - Status type
 */
export function showStatus(message, type = 'info') {
    const status = document.getElementById('status');
    status.textContent = message;
    status.className = `status ${type} visible`;
}

/**
 * Creates update notification
 * @param {string} message - Update message
 */
export function createUpdateNotification(message) {
    const container = document.getElementById('update-notifications');
    const notification = document.createElement('div');
    notification.className = 'update-notification';
    notification.textContent = message;
    container.appendChild(notification);
}

/**
 * Updates the tree visualization
 * @param {Object} fileSystem - File system structure
 */
export function updateTreeVisualization(fileSystem) {
    const treeContainer = document.getElementById('tree-container');
    
    function renderNode(node) {
        const attrs = [];
        if (node.isSymlink) {
            attrs.push(`data-symlink="${node.symlink}"`);
        }
        if (node.size) {
            attrs.push(`data-size="${node.size}"`);
        }
        
        return `
            <div class="tree-node" data-path="${node.path}" ${attrs.join(' ')}>
                <span class="node-name">${node.path}</span>
                ${node.children?.map(child => renderNode(child)).join('') || ''}
            </div>
        `;
    }
    
    treeContainer.innerHTML = renderNode(fileSystem);
}
