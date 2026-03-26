/**
 * Mock Performance API for testing
 */
class PerformanceMock {
    /** @type {Map<string, number>} */
    #marks = new Map();
    /** @type {Map<string, {duration: number}>} */
    #measures = new Map();

    /**
     * Clears all performance marks
     */
    clearMarks() {
        this.#marks.clear();
    }

    /**
     * Clears all performance measurements
     */
    clearMeasures() {
        this.#measures.clear();
    }

    /**
     * Creates a performance mark
     * @param {string} name - Name of the mark
     */
    mark(name) {
        this.#marks.set(name, Date.now());
    }

    /**
     * Creates a performance measure between two marks
     * @param {string} name - Name of the measure
     * @param {string} startMark - Start mark name
     * @param {string} endMark - End mark name
     */
    measure(name, startMark, endMark) {
        const start = this.#marks.get(startMark) ?? 0;
        const end = this.#marks.get(endMark) ?? Date.now();
        this.#measures.set(name, { duration: end - start });
    }

    /**
     * Gets entries by name
     * @param {string} name - Name of the entry
     * @returns {Array<{duration: number}>} Array of matching entries
     */
    getEntriesByName(name) {
        const entry = this.#measures.get(name);
        return entry ? [entry] : [];
    }

    /**
     * Gets current high-resolution timestamp
     * @returns {number} Current timestamp
     */
    now() {
        return Date.now();
    }
}

/**
 * Setup performance mock
 * @returns {void}
 */
export function setupPerformanceMock() {
    const mock = new PerformanceMock();
    Object.defineProperty(global, 'performance', {
        value: mock,
        writable: true,
        configurable: true
    });
}
