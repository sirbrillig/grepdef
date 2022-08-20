// @ts-check
const getJsRegexp = require('./lang/js.js');
const getPhpRegexp = require('./lang/php.js');
const ripgrepSearch = require('./searchers/ripgrep.js');
const humanOutput = require('./reporters/human.js');
const fs = require('fs');
const util = require('util');

const lstat = util.promisify(fs.lstat);
const readdir = util.promisify(fs.readdir);

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
 * @typedef {(arg: SearchResult[], config: SearchConfig) => string[]} ReporterFunction
 */

/**
 * @typedef {(regexp: string, config: SearchConfig) => Promise<SearchResult[]>} SearchFunction
 */

/**
 * @typedef {object} SearchConfig
 * @property {FileType|null} [type]
 * @property {boolean} [verbose]
 * @property {boolean} [showLineNumbers]
 * @property {boolean} [disableColor]
 * @property {SearchTool} searchTool
 * @property {Glob} path
 */

/**
 * @typedef {object} SearchResult
 * @property {string} path
 * @property {number} line
 * @property {string} text
 */

/**
 * @param {string} symbol - the symbol to search for
 * @param {SearchConfig} config
 * @param {ReporterType} reporterName
 * @returns void
 */
async function searchAndReport(symbol, config, reporterName) {
	const results = await search(symbol, config);
	const reporter = getReporterForReporterName(reporterName);
	const output = reporter(results, config);
	output.forEach(result => console.log(result));
}

/**
 * @param {string} symbol - the symbol to search for
 * @param {SearchConfig} config
 * @returns {Promise<SearchResult[]>}
 */
async function search(symbol, { type, searchTool, path, ...otherOptions }) {
	if (!type) {
		type = await guessType(path);
	}
	const regexp = getRegexpForType(symbol, type);
	const searcher = getSearchToolForSearcherName(searchTool);
	return searcher(regexp, { type, searchTool, path, ...otherOptions });
}

/**
 * @param {SearchTool} name
 * @returns {SearchFunction}
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
			throw new Error(`Unknown language type '${type}'`);
	}
}

/**
 * @param {string} type
 * @returns {FileType}
 */
function normalizeType(type) {
	switch (type) {
		case 'javascript':
		case 'javascript.jsx':
		case 'javascriptreact':
		case 'typescript':
		case 'typescriptreact':
		case 'typescript.tsx':
		case 'js':
			return 'js';
		case 'php':
			return 'php';
		default:
			throw new Error(`Unknown language type '${type}'`);
	}
}

/**
 * @param {ReporterType} type
 * @returns {ReporterFunction}
 */
function getReporterForReporterName(type) {
	switch (type) {
		case 'human':
			return humanOutput;
		default:
			throw new Error('Unknown reporter type');
	}
}

/**
 * @param {Glob} path
 * @returns {Promise<FileType|null>}
 */
async function guessType(path) {
	const pathNodes = path.split(' ');
	const guess = guessTypeFromFiles(pathNodes);
	return guess || guessTypeFromNearby(path);
}

/**
 * @param {string[]} files
 * @returns {FileType|null}
 */
function guessTypeFromFiles(files) {
	const jsFileRegexp = /\.(js|ts|jsx|tsx)$/;
	if (files.some(node => jsFileRegexp.test(node))) {
		return 'js';
	}
	if (files.some(node => node.endsWith('.php'))) {
		return 'php';
	}
	if (files.some(node => node === 'package.json')) {
		return 'js';
	}
	return null;
}

/**
 * @param {Glob} path
 * @returns {Promise<FileType|null>}
 */
async function guessTypeFromNearby(path) {
	const pathNodes = path.split(' ');
	// look for any js/php files in the current directory
	for (const node of pathNodes) {
		const nodeStat = await lstat(node);
		if (nodeStat.isDirectory()) {
			const nodesInDirectory = await readdir(node);
			const guess = guessTypeFromFiles(nodesInDirectory);
			if (guess) {
				return guess;
			}
		}
	}
	return null;
}

module.exports = {
	search,
	searchAndReport,
	normalizeType,
	getReporterForReporterName,
};
