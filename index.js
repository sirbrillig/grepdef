// @format

const minimist = require('minimist');
const getJsRegexp = require('./src/lang/js.js');
const getPhpRegexp = require('./src/lang/php.js');
const ripgrepSearch = require('./src/searchers/ripgrep.js');
const humanOutput = require('./src/reporters/human.js');

async function search({ symbol, type, verbose, reporterName, searchTool, path }) {
	const regexp = getRegexpForType(symbol, type);
	const results = await getSearchToolForSearcherName(searchTool)({ regexp, type, verbose, path });
	getReporterForReporterName(reporterName)(results);
}

function getSearchToolForSearcherName(name) {
	switch (name) {
		case 'ripgrep':
			return ripgrepSearch;
		default:
			throw new Error('Unknown search tool');
	}
}

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

function getReporterForReporterName(type) {
	switch (type) {
		case 'human':
			return humanOutput;
		default:
			throw new Error('Unknown reporter type');
	}
}

function printHelp() {
	const helpText = `
grepdef: search for symbol definitions in various programming languages

Usage: grepdef --type <type> <symbol> [path]

The type is a vim-compatible filetype. One of 'js', 'php', or an alias for
those strings (eg: 'javascript.jsx').

The symbol is the full string name of a class, function, variable, or similar
construct.

If a search path is not provided, this will search starting from the current
directory.

The output is like using grep, but will only show places where that symbol is
defined (no partial matches, variable uses, or function calls). The search uses
a regular expression so it is unaware of scope and is far from fullproof, but
should be easier than a grep by itself.
	`;
	console.log(helpText);
}

async function main(args) {
	const options = minimist(args);
	const langType = options.type;
	const searchTool = options.searcher || 'ripgrep';
	const reporterName = options.reporter || 'human';
	const searchSymbol = options._[0];
	const path = options._[1] || '.';
	if (options.h || options.help) {
		printHelp();
		process.exit(0);
		return;
	}
	if (!searchSymbol) {
		console.error('No search symbol provided.');
		printHelp();
		process.exit(1);
		return;
	}
	search({
		symbol: searchSymbol,
		type: normalizeType(langType),
		verbose: !!options.verbose,
		reporterName,
		searchTool,
		path,
	}).catch(error => {
		console.error(error.message);
		printHelp();
		process.exit(1);
	});
}

module.exports = main;
