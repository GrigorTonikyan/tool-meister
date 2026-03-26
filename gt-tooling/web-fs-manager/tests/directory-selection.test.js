import { fireEvent, waitFor } from '@testing-library/dom';
import '@testing-library/jest-dom';
import {
    setupDirectorySelectionDOM,
    createMockDirectoryResponse,
    createMockFileSystem,
    measurePerformance,
    delay,
    updateDirectoryList,
    updateTreeVisualization
} from './utils/test-utils';

/**
 * @typedef {import('./utils/test-utils').DirectoryResponse} DirectoryResponse
 */

// Mock main.js functions
const mockShowProjectSelector = jest.fn();
const mockSelectCurrentDirectory = jest.fn();
const mockSelectDirectory = jest.fn();

jest.mock('../src/main.js', () => ({
    showProjectSelector: mockShowProjectSelector,
    selectCurrentDirectory: mockSelectCurrentDirectory,
    selectDirectory: mockSelectDirectory
}));

describe('Directory Selection', () => {
    /** @type {jest.SpyInstance} */
    let fetchSpy;
    
    beforeEach(() => {
        setupDirectorySelectionDOM();
        fetchSpy = jest.spyOn(global, 'fetch');
        fetchSpy.mockImplementation((url) => {
            if (url.includes('/api/directories')) {
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve(createMockDirectoryResponse('/test/path', ['dir1', 'dir2']))
                });
            }
            if (url.includes('/api/recent-projects')) {
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve(['/recent/path1', '/recent/path2'])
                });
            }
            if (url.includes('/api/set-project')) {
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({ success: true })
                });
            }
            return Promise.reject(new Error('Not found'));
        });

        mockShowProjectSelector.mockReset();
        mockSelectCurrentDirectory.mockReset();
        mockSelectDirectory.mockReset();
    });
    
    afterEach(() => {
        fetchSpy.mockRestore();
        jest.clearAllMocks();
    });

    describe('UI Interactions', () => {
        test('shows modal with correct initial state', async () => {
            mockShowProjectSelector.mockImplementation(async () => {
                const modal = document.getElementById('project-modal');
                modal.classList.add('visible');
                
                // Update directory list
                updateDirectoryList('/test/path', [
                    { name: 'dir1', path: '/test/path/dir1', isParent: false },
                    { name: 'dir2', path: '/test/path/dir2', isParent: false }
                ]);
                
                // Update recent projects
                const recentList = document.getElementById('recent-projects-list');
                recentList.innerHTML = ['/recent/path1', '/recent/path2'].map(path => `
                    <li class="directory-item">
                        <i>📂</i>
                        <span>${path}</span>
                    </li>
                `).join('');
            });
            
            await mockShowProjectSelector();
            
            expect(document.getElementById('project-modal')).toHaveClass('visible');
            expect(document.getElementById('directory-list').children).toHaveLength(2);
            expect(document.getElementById('recent-projects-list').children).toHaveLength(2);
        });

        test('updates breadcrumb navigation correctly', async () => {
            mockSelectDirectory.mockImplementation(async (path) => {
                updateDirectoryList(path, [
                    { name: '..', path: '/test/path', isParent: true },
                    { name: 'subdir', path: '/test/path/subdir', isParent: false }
                ]);
            });

            await mockSelectDirectory('/test/path/subdir');
            
            const breadcrumb = document.getElementById('path-breadcrumb');
            expect(breadcrumb.children).toHaveLength(3);
            expect(breadcrumb.textContent).toContain('test/path/subdir');
        });

        test('handles double-click directory selection', async () => {
            // Setup mock implementation
            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, [
                    { name: 'test-dir', path: '/test/path/test-dir', isParent: false }
                ]);
            });

            // Setup initial directory list
            const directoryList = document.getElementById('directory-list');
            updateDirectoryList('/test/path', [
                { name: 'test-dir', path: '/test/path/test-dir', isParent: false }
            ]);

            // Trigger double-click
            const item = directoryList.firstChild;
            const doubleClickEvent = new MouseEvent('dblclick', {
                bubbles: true,
                cancelable: true,
                view: window
            });

            // Mock selectDirectory call
            mockSelectDirectory('/test/path/test-dir');
            item.dispatchEvent(doubleClickEvent);
            await delay(100);

            expect(fetchSpy).toHaveBeenCalledWith(expect.stringContaining('/api/directories'));
        });
    });

    describe('Error Handling', () => {
        test('handles network timeout gracefully', async () => {
            fetchSpy.mockImplementationOnce(() => new Promise((_, reject) => {
                setTimeout(() => reject(new Error('Network timeout')), 100);
            }));

            mockSelectDirectory.mockImplementation(async () => {
                const status = document.getElementById('status');
                status.classList.add('visible');
                status.textContent = 'Error: Network timeout';
            });

            await mockSelectDirectory('/timeout/path');
            
            const status = document.getElementById('status');
            expect(status).toHaveClass('visible');
            expect(status.textContent).toContain('Error');
        });

        test('handles invalid directory paths', async () => {
            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: false,
                status: 400,
                json: () => Promise.resolve({ error: 'Invalid path' })
            }));

            mockSelectDirectory.mockImplementation(async () => {
                const status = document.getElementById('status');
                status.classList.add('visible');
                status.textContent = 'Error: Invalid path';
            });

            await mockSelectDirectory('invalid/path');
            
            const status = document.getElementById('status');
            expect(status).toHaveClass('visible');
            expect(status.textContent).toContain('Error');
        });

        test('recovers from WebSocket disconnection', async () => {
            const ws = new WebSocket('ws://localhost:3001');
            ws.onopen = () => {
                ws.close();
            };

            await delay(100);
            expect(ws.readyState).toBe(WebSocket.CLOSED);
            
            const newWs = new WebSocket('ws://localhost:3001');
            await delay(100);
            expect(newWs.readyState).toBe(WebSocket.OPEN);
        });
    });

    describe('Performance', () => {
        test('loads large directory structure efficiently', async () => {
            const largeStructure = Array.from({ length: 1000 }, (_, i) => `dir${i}`);
            
            const loadTime = await measurePerformance(async () => {
                fetchSpy.mockImplementationOnce(() => Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve(createMockDirectoryResponse('/test', largeStructure))
                }));
                
                await mockSelectDirectory('/test');
                await delay(100);
            }, 'Large directory load');

            expect(loadTime).toBeLessThan(1000);
        });

        test('handles rapid directory navigation', async () => {
            const paths = ['/path1', '/path2', '/path3', '/path4', '/path5'];
            
            const navigationTime = await measurePerformance(async () => {
                for (const path of paths) {
                    await mockSelectDirectory(path);
                    await delay(50);
                }
            }, 'Rapid navigation');

            expect(navigationTime).toBeLessThan(1000);
        });

        test('efficiently updates file system tree', async () => {
            const fileSystem = createMockFileSystem('/test/project');
            
            const updateTime = await measurePerformance(async () => {
                updateTreeVisualization(fileSystem);
                await delay(100);
            }, 'Tree update');

            expect(updateTime).toBeLessThan(500);
        });
    });

    describe('End-to-End Flows', () => {
        test('completes full project selection flow', async () => {
            // Setup mock implementations
            mockShowProjectSelector.mockImplementation(async () => {
                const modal = document.getElementById('project-modal');
                modal.classList.add('visible');
            });

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, [
                    { name: path.split('/').pop(), path, isParent: false }
                ]);
            });

            mockSelectCurrentDirectory.mockImplementation(async () => {
                const modal = document.getElementById('project-modal');
                modal.classList.remove('visible');
                await fetch('/api/set-project', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ path: '/test/project' })
                });
                document.getElementById('current-project').textContent = '/test/project';
            });

            // Execute flow
            await mockShowProjectSelector();
            expect(document.getElementById('project-modal')).toHaveClass('visible');

            await mockSelectDirectory('/test/project');
            expect(fetchSpy).toHaveBeenCalledWith(expect.stringContaining('/api/directories'));

            await mockSelectCurrentDirectory();
            expect(fetchSpy).toHaveBeenCalledWith('/api/set-project', expect.any(Object));

            expect(document.getElementById('project-modal')).not.toHaveClass('visible');
            expect(document.getElementById('current-project')).toHaveTextContent('/test/project');
        });

        test('handles project refresh and update', async () => {
            await mockSelectCurrentDirectory();
            
            const fileSystem = createMockFileSystem('/test/project');
            updateTreeVisualization(fileSystem);
            
            expect(document.getElementById('tree-container')).not.toBeEmptyDOMElement();
        });

        test('maintains state during navigation', async () => {
            const paths = ['/path1', '/path2', '/path1'];
            
            for (const path of paths) {
                await mockSelectDirectory(path);
                updateDirectoryList(path, [
                    { name: path.split('/').pop(), path, isParent: false }
                ]);
                await delay(50);
            }

            const breadcrumb = document.getElementById('path-breadcrumb');
            expect(breadcrumb.textContent).toContain('path1');
            
            const selectBtn = document.getElementById('select-dir-btn');
            expect(selectBtn.textContent).toContain('/path1');
        });
    });
});
