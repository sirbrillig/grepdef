// @ts-check
const getJsRegexp = require('./lang/js.js');
const getPhpRegexp = require('./lang/php.js');
const ripgrepSearch = require('./searchers/ripgrep.js');
const humanOutput = require('./reporters/human.js');

/**
 * @typedef {'js'|'php'} FileType
 */

/**
 * @typedef {'human'} ReporterType
 */

/**
 * @typedef {'ripgrep'} SearchTool
 */

/**
 * @typedef {string} Glob
 */

/**
 * @typedef {object} SearchConfig
 * @property {string} symbol - the symbol to search for
 * @property {FileType} type
 * @property {boolean} verbose
 * @property {SearchTool} searchTool
 * @property {Glob} path
 */

/**
 * @param {SearchConfig} config
 * @param {ReporterType} reporterName
 * @returns void
 */
async function searchAndReport({ symbol, type, verbose, searchTool, path }, reporterName) {
	const results = await search({ symbol, type, verbose, searchTool, path })
	const reporter = getReporterForReporterName(reporterName);
	reporter(results);
}

/**
 * @param {SearchConfig} config
 * @returns void
 */
async function search({ symbol, type, verbose, searchTool, path }) {
	const regexp = getRegexpForType(symbol, type);
	const searcher = getSearchToolForSearcherName(searchTool);
	return searcher({ regexp, type, verbose, path });
}

/**
 * @param {SearchTool} name
 * @returns {function}
 */
function getSearchToolForSearcherName(name) {
	switch (name) {
		case 'ripgrep':
			return ripgrepSearch;
		default:
			throw new Error('Unknown search tool');
	}
}

/**
 * @param {string} symbol - the symbol to search for
 * @param {FileType} type
 * @returns {string} the regexp to use
 */
function getRegexpForType(symbol, type) {
	switch (type) {
		case 'js':
			return getJsRegexp(symbol);
		case 'php':
			return getPhpRegexp(symbol);
		default:
			throw new Error('Unknown language type');
	}
}

/**
 * @param {string} type
 * @returns {FileType|null}
 */
function normalizeType(type) {
	switch (type) {
		case 'javascript':
		case 'javascript.jsx':
		case 'javascriptreact':
		case 'typescript':
		case 'js':
			return 'js';
		case 'php':
			return 'php';
		default:
			return null;
	}
}

/**
 * @param {ReporterType} type
 * @returns {function}
 */
function getReporterForReporterName(type) {
	switch (type) {
		case 'human':
			return humanOutput;
		default:
			throw new Error('Unknown reporter type');
	}
}

module.exports = {
	search,
	searchAndReport,
	normalizeType,
};
