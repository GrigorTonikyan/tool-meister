import '@testing-library/jest-dom';
import {
    createDeepDirectory,
    createLargeDirectory,
    createSymlinkStructure,
    generateLongPath,
    getSpecialPaths,
    simulateFileSystemEvents,
    simulateMemoryPressure,
    simulateNetwork
} from './utils/edge-case-utils';
import { setupPerformanceMock } from './utils/performance-mock';
import {
    createUpdateNotification,
    delay,
    setupDirectorySelectionDOM,
    showStatus,
    updateDirectoryList,
    updateTreeVisualization
} from './utils/test-utils';

// Setup performance mock
setupPerformanceMock();

describe('Edge Cases', () => {
    /** @type {jest.SpyInstance} */
    let fetchSpy;
    /** @type {jest.Mock} */
    let mockShowProjectSelector;
    /** @type {jest.Mock} */
    let mockSelectDirectory;

    beforeEach(() => {
        setupDirectorySelectionDOM();
        fetchSpy = jest.spyOn(global, 'fetch');
        mockShowProjectSelector = jest.fn();
        mockSelectDirectory = jest.fn();

        // Reset performance metrics
        performance.clearMarks();
        performance.clearMeasures();
    });

    afterEach(() => {
        fetchSpy.mockRestore();
        jest.clearAllMocks();
    });

    describe('Path Handling', () => {
        test('handles extremely long paths', async () => {
            const longPath = generateLongPath(249);
            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: true,
                json: () => Promise.resolve({ path: longPath, items: [] })
            }));

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, []);
            });

            await mockSelectDirectory(longPath);
            const breadcrumb = document.getElementById('path-breadcrumb');

            expect(breadcrumb.textContent.length).toBe(249);
            expect(fetchSpy).toHaveBeenCalledWith(expect.stringContaining(encodeURIComponent(longPath)));
        });

        test('handles special characters in paths', async () => {
            const specialPaths = getSpecialPaths();

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, []);
            });

            for (const path of specialPaths) {
                fetchSpy.mockImplementationOnce(() => Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({ path, items: [] })
                }));

                await mockSelectDirectory(path);
                expect(fetchSpy).toHaveBeenCalledWith(expect.stringContaining(encodeURIComponent(path)));
            }
        });

        test('handles relative path navigation', async () => {
            const paths = ['.', '..', '../..', './subfolder'];
            const baseDir = '/test/base/dir';

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(baseDir, []);
            });

            for (const path of paths) {
                fetchSpy.mockImplementationOnce(() => Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({ path: baseDir, items: [] })
                }));

                await mockSelectDirectory(path);
                expect(fetchSpy).toHaveBeenCalled();
            }
        });
    });

    describe('Directory Structure', () => {
        test('handles empty directories', async () => {
            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: true,
                json: () => Promise.resolve({ path: '/empty', items: [] })
            }));

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, []);
            });

            await mockSelectDirectory('/empty');
            const directoryList = document.getElementById('directory-list');

            expect(directoryList.children.length).toBe(0);
            expect(directoryList).toBeEmptyDOMElement();
        });

        test('handles deeply nested directories', async () => {
            const deepStructure = createDeepDirectory(20);
            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: true,
                json: () => Promise.resolve({ path: deepStructure.path, items: deepStructure.children })
            }));

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateTreeVisualization(deepStructure);
            });

            await mockSelectDirectory(deepStructure.path);

            const treeContainer = document.getElementById('tree-container');
            expect(treeContainer.querySelectorAll('.tree-node').length).toBeGreaterThan(19);
        });

        test('handles large number of files', async () => {
            const largeDirectory = createLargeDirectory(10000);

            performance.mark('start-large-dir');

            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: true,
                json: () => Promise.resolve({ path: '/large', items: largeDirectory })
            }));

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, largeDirectory);
            });

            await mockSelectDirectory('/large');

            performance.mark('end-large-dir');
            performance.measure('large-dir-load', 'start-large-dir', 'end-large-dir');

            const measure = performance.getEntriesByName('large-dir-load')[0];
            expect(measure.duration).toBeLessThan(1000);

            const directoryList = document.getElementById('directory-list');
            expect(directoryList.children.length).toBe(10000);
        });

        test('handles symbolic links', async () => {
            const symlinkStructure = createSymlinkStructure('/test');
            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: true,
                json: () => Promise.resolve({ path: symlinkStructure.path, items: symlinkStructure.children })
            }));

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateTreeVisualization(symlinkStructure);
            });

            await mockSelectDirectory(symlinkStructure.path);

            const treeContainer = document.getElementById('tree-container');
            const symlinks = treeContainer.querySelectorAll('[data-symlink]');
            expect(symlinks.length).toBeGreaterThan(0);
        });
    });

    describe('Network Conditions', () => {
        test('handles high latency', async () => {
            fetchSpy.mockImplementation(async () => {
                await simulateNetwork(1000);
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({ path: '/test', items: [] })
                });
            });

            mockSelectDirectory.mockImplementation(async (path) => {
                showStatus('Loading...', 'info');
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                showStatus('Directory loaded', 'success');
            });

            const start = Date.now();
            await mockSelectDirectory('/test');
            const end = Date.now();

            expect(end - start).toBeGreaterThanOrEqual(1000);
            expect(document.getElementById('status')).toHaveClass('visible');
        });

        test('handles intermittent failures', async () => {
            fetchSpy.mockImplementation(async () => {
                await simulateNetwork(100, 0.5);
                return Promise.resolve({
                    ok: true,
                    json: () => Promise.resolve({ path: '/test', items: [] })
                });
            });

            mockSelectDirectory.mockImplementation(async (path) => {
                try {
                    await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                    showStatus('Success', 'success');
                } catch (error) {
                    showStatus('Network error', 'error');
                    throw error;
                }
            });

            let successCount = 0;
            const attempts = 10;

            for (let i = 0; i < attempts; i++) {
                try {
                    await mockSelectDirectory('/test');
                    successCount++;
                } catch (error) {
                    expect(error.message).toBe('Network error');
                }
            }

            expect(successCount).toBeLessThan(attempts);
        });
    });

    describe('Concurrent Operations', () => {
        test('handles rapid directory changes', async () => {
            const paths = Array.from({ length: 10 }, (_, i) => `/path${i}`);

            mockSelectDirectory.mockImplementation(async (path) => {
                await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                updateDirectoryList(path, []);
            });

            const promises = paths.map(path => mockSelectDirectory(path));
            await Promise.all(promises);

            const currentPath = document.getElementById('path-breadcrumb').getAttribute('data-path');
            expect(paths).toContain(currentPath);
        });

        test('handles file system updates during navigation', async () => {
            const ws = new WebSocket('ws://localhost:3001');
            await mockSelectDirectory('/test');

            // Setup WebSocket event handling
            ws.onmessage = (event) => {
                createUpdateNotification(`Directory changed: ${event.data}`);
            };

            // Simulate file system events
            simulateFileSystemEvents(ws, '/test', 100);
            await delay(1000);

            const updates = document.querySelectorAll('.update-notification');
            expect(updates.length).toBeGreaterThan(0);
        });
    });

    describe('Resource Constraints', () => {
        test('handles memory pressure', async () => {
            performance.mark('start-memory');

            simulateMemoryPressure(100);
            const largeDirectory = createLargeDirectory(50000);
            await updateDirectoryList('/large', largeDirectory);

            performance.mark('end-memory');
            performance.measure('memory-test', 'start-memory', 'end-memory');

            const measure = performance.getEntriesByName('memory-test')[0];
            expect(measure.duration).toBeLessThan(5000);
        });
    });

    describe('Error Handling', () => {
        test('handles malformed responses', async () => {
            fetchSpy.mockImplementationOnce(() => Promise.resolve({
                ok: true,
                json: () => Promise.reject(new Error('Invalid JSON'))
            }));

            mockSelectDirectory.mockImplementation(async (path) => {
                try {
                    const response = await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                    await response.json();
                } catch (error) {
                    showStatus('Error: Invalid response format', 'error');
                    throw error;
                }
            });

            try {
                await mockSelectDirectory('/test');
            } catch (error) {
                const status = document.getElementById('status');
                expect(status).toHaveClass('visible');
                expect(status.textContent).toMatch(/error/i);
            }
        });

        test('handles server errors', async () => {
            const errorCodes = [400, 401, 403, 404, 500, 502, 503];

            for (const status of errorCodes) {
                fetchSpy.mockImplementationOnce(() => Promise.resolve({
                    ok: false,
                    status,
                    json: () => Promise.resolve({ error: `Server error ${status}` })
                }));

                mockSelectDirectory.mockImplementation(async (path) => {
                    try {
                        const response = await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                        const data = await response.json();
                        if (!response.ok) {
                            showStatus(`Error: ${data.error}`, 'error');
                            throw new Error(data.error);
                        }
                    } catch (error) {
                        showStatus(`Error: ${error.message}`, 'error');
                        throw error;
                    }
                });

                try {
                    await mockSelectDirectory('/test');
                } catch (error) {
                    const status = document.getElementById('status');
                    expect(status).toHaveClass('visible');
                    expect(status.textContent).toMatch(/error/i);
                    expect(status.textContent).toMatch(new RegExp(status.toString()));
                }
            }
        });

        test('handles invalid paths', async () => {
            const invalidPaths = [
                '',                    // Empty path
                '/',                   // Root only
                '//',                  // Double slash
                '/..',                 // Parent of root
                '/../../etc/passwd',   // Path traversal attempt
                'C:\\Windows\\System32' // Windows system directory
            ];

            for (const path of invalidPaths) {
                fetchSpy.mockImplementationOnce(() => Promise.resolve({
                    ok: false,
                    status: 400,
                    json: () => Promise.resolve({ error: 'Invalid path' })
                }));

                mockSelectDirectory.mockImplementation(async (path) => {
                    try {
                        const response = await fetch(`/api/directories?path=${encodeURIComponent(path)}`);
                        const data = await response.json();
                        if (!response.ok) {
                            showStatus(`Error: ${data.error}`, 'error');
                            throw new Error(data.error);
                        }
                    } catch (error) {
                        showStatus(`Error: ${error.message}`, 'error');
                        throw error;
                    }
                });

                try {
                    await mockSelectDirectory(path);
                } catch (error) {
                    const status = document.getElementById('status');
                    expect(status).toHaveClass('visible');
                    expect(status.textContent).toMatch(/error/i);
                    expect(status.textContent).toMatch(/invalid/i);
                }
            }
        });
    });
});
