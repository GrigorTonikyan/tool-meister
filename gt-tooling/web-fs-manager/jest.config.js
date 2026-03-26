/** @type {import('jest').Config} */
export default {
    testEnvironment: 'jsdom',
    transform: {
        '^.+\\.js$': ['babel-jest', { configFile: './babel.config.js' }]
    },
    moduleNameMapper: {
        '\\.(css|less|scss|sass)$': 'identity-obj-proxy'
    },
    setupFilesAfterEnv: ['./tests/setup.js'],
    transformIgnorePatterns: [
        'node_modules/(?!(node-fetch)/)'
    ],
    moduleFileExtensions: ['js', 'json'],
    testMatch: ['**/__tests__/**/*.js', '**/?(*.)+(spec|test).js'],
    verbose: true
};
