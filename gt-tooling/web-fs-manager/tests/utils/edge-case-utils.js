/**
 * @typedef {import('./test-utils').DirectoryItem} DirectoryItem
 */

/**
 * Generates a long path
 * @param {number} length - Target length of the path
 * @returns {string} Long path
 */
export function generateLongPath(length) {
    const parts = Math.floor((length - 1) / 10); // Account for slashes
    const chars = 'abcdefghijklmnopqrstuvwxyz';
    const segments = Array.from({ length: parts }, (_, i) => {
        let segment = '';
        while (segment.length < 9) {
            segment += chars[Math.floor(Math.random() * chars.length)];
        }
        return segment;
    });
    return '/' + segments.join('/');
}

/**
 * Creates a deep directory structure
 * @param {number} depth - Number of nested levels
 * @returns {Object} Nested directory structure
 */
export function createDeepDirectory(depth) {
    let current = { path: '/root', children: [] };
    let parent = current;
    
    for (let i = 1; i < depth; i++) {
        const child = {
            path: `${parent.path}/level${i}`,
            children: []
        };
        parent.children.push(child);
        parent = child;
    }
    
    return current;
}

/**
 * Simulates network conditions
 * @param {number} latency - Simulated latency in ms
 * @param {number} [errorRate=0] - Rate of errors (0-1)
 * @returns {Promise<void>}
 */
export async function simulateNetwork(latency, errorRate = 0) {
    await new Promise((resolve, reject) => {
        setTimeout(() => {
            if (Math.random() < errorRate) {
                reject(new Error('Network error'));
            } else {
                resolve();
            }
        }, latency);
    });
}

/**
 * Creates a large directory listing
 * @param {number} count - Number of items
 * @returns {DirectoryItem[]} Array of directory items
 */
export function createLargeDirectory(count) {
    return Array.from({ length: count }, (_, i) => ({
        name: `item${i.toString().padStart(5, '0')}`,
        path: `/large/item${i.toString().padStart(5, '0')}`,
        isParent: false
    }));
}

/**
 * Simulates file system events through WebSocket
 * @param {WebSocket} ws - WebSocket instance
 * @param {string} basePath - Base path for events
 * @param {number} count - Number of events to simulate
 */
export function simulateFileSystemEvents(ws, basePath, count) {
    const events = [
        'created',
        'modified',
        'deleted',
        'renamed'
    ];

    for (let i = 0; i < count; i++) {
        const event = events[Math.floor(Math.random() * events.length)];
        const path = `${basePath}/file${i}.txt`;
        
        // Simulate WebSocket message
        if (ws.onmessage) {
            ws.onmessage({ data: JSON.stringify({
                type: event,
                path: path,
                timestamp: Date.now()
            })});
        }
    }
}

/**
 * Creates a directory structure with symlinks
 * @param {string} basePath - Base path for structure
 * @returns {Object} Directory structure with symlinks
 */
export function createSymlinkStructure(basePath) {
    return {
        path: basePath,
        children: [
            {
                path: `${basePath}/real`,
                children: [
                    { path: `${basePath}/real/file.txt`, size: 100 }
                ]
            },
            {
                path: `${basePath}/link`,
                symlink: `${basePath}/real`,
                isSymlink: true
            }
        ]
    };
}

/**
 * Generates paths with special characters
 * @returns {string[]} Array of special paths
 */
export function getSpecialPaths() {
    return [
        '/path with spaces/file.txt',
        '/path#with#hashes/file.txt',
        '/path@with@at/file.txt',
        '/path[with]brackets/file.txt',
        '/path(with)parentheses/file.txt',
        '/path&with&ampersands/file.txt',
        '/路径/测试.txt',
        '/путь/тест.txt',
        '/パス/テスト.txt',
        '/경로/테스트.txt',
        '/🌟/✨/💫.txt'
    ];
}

/**
 * Simulates memory pressure
 * @param {number} targetMB - Target memory usage in MB
 */
export function simulateMemoryPressure(targetMB) {
    const array = [];
    const megabyte = 1024 * 1024;
    while (process.memoryUsage().heapUsed < targetMB * megabyte) {
        array.push(new Array(1024).fill('x').join(''));
    }
}

/**
 * Creates a mock error response
 * @param {number} status - HTTP status code
 * @param {string} message - Error message
 * @returns {Response} Mock Response object
 */
export function createErrorResponse(status, message) {
    return new Response(
        JSON.stringify({ error: message }),
        {
            status,
            headers: { 'Content-Type': 'application/json' }
        }
    );
}
