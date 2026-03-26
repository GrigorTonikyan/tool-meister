export default {
    presets: [
        ['@babel/preset-env', {
            targets: { node: 'current' },
            modules: 'auto'
        }]
    ],
    assumptions: {
        setPublicClassFields: true
    }
};
