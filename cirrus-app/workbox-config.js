module.exports = {
  globDirectory: 'dist/',
  globPatterns: ['**/*.{woff,woff2,js,css,png,jpg,svg,html}'],
  /* pass array of globs to exclude from caching */
  globIgnores: [],
  ignoreURLParametersMatching: [/^utm_/, /^fbclid$/],
  swDest: 'dist/service-worker.js',
};
