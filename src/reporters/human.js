// @ts-check

const chalk = require('chalk');

/**
 * @param {import('../searchers/ripgrep').SearchResult[]} results
 * @param {import('../general').SearchConfig} config
 * @return {string[]}
 */
function outputResults(results, {showLineNumbers}) {
	return results.map(match => {
		if (showLineNumbers) {
			return `${chalk.default.magenta(match.path)}:${chalk.default.green(String(match.line))}:${match.text}`;
		}
		return `${chalk.default.magenta(match.path)}:${match.text}`;
	});
}

module.exports = outputResults;
