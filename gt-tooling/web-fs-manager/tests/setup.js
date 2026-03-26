import '@testing-library/jest-dom';
import { TextEncoder, TextDecoder } from 'util';

// Mock fetch globally
global.fetch = jest.fn();
global.Request = jest.fn();
global.Response = jest.fn();

// Mock TextEncoder/Decoder
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder;

// Mock d3
jest.mock('d3', () => ({
    hierarchy: jest.fn(() => ({
        descendants: () => [],
        links: () => []
    })),
    tree: jest.fn(() => ({
        size: jest.fn().mockReturnThis(),
        call: jest.fn()
    })),
    select: jest.fn(() => ({
        append: jest.fn().mockReturnThis(),
        attr: jest.fn().mockReturnThis(),
        style: jest.fn().mockReturnThis(),
        text: jest.fn().mockReturnThis(),
        data: jest.fn().mockReturnThis(),
        enter: jest.fn().mockReturnThis(),
        selectAll: jest.fn().mockReturnThis(),
        remove: jest.fn().mockReturnThis(),
        on: jest.fn().mockReturnThis(),
        classed: jest.fn().mockReturnThis()
    })),
    linkHorizontal: jest.fn(() => ({
        x: jest.fn().mockReturnThis(),
        y: jest.fn().mockReturnThis()
    }))
}));

// Mock WebSocket
class MockWebSocket {
    constructor(url) {
        this.url = url;
        this.readyState = WebSocket.CONNECTING;
        setTimeout(() => {
            this.readyState = WebSocket.OPEN;
            if (this.onopen) this.onopen();
        }, 0);
    }

    send(data) {
        // Mock send
    }

    close() {
        this.readyState = WebSocket.CLOSED;
        if (this.onclose) this.onclose();
    }
}

global.WebSocket = MockWebSocket;
WebSocket.CONNECTING = 0;
WebSocket.OPEN = 1;
WebSocket.CLOSING = 2;
WebSocket.CLOSED = 3;
