// @format

const minimist = require('minimist');
const getJsRegexp = require('./src/lang/js.js');
const getPhpRegexp = require('./src/lang/php.js');
const searchWithRegexp = require('./src/searchers/ripgrep.js');
const humanOutput = require('./src/reporters/human.js');

async function search({ symbol, type, verbose, reporterName, path }) {
	const regexp = getRegexpForType(symbol, type);
	const results = await searchWithRegexp({ regexp, type, verbose, path });
	getReporterForReporterName(reporterName)(results);
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

function getReporterForReporterName(type) {
	switch (type) {
		case 'human':
			return humanOutput;
		default:
			throw new Error('Unknown reporter type');
	}
}

async function main(args) {
	const options = minimist(args);
	const langType = options.type;
	const reporterName = options.reporter || 'human';
	const searchSymbol = options._[0];
	const path = options._[1] || '.';
	if (!searchSymbol) {
		console.error('No search symbol provided.');
		process.exit(1);
	}
	search({
		symbol: searchSymbol,
		type: langType,
		verbose: !!options.verbose,
		reporterName,
		path,
	}).catch(error => {
		console.error(error.message);
		process.exit(1);
	});
}

module.exports = main;
