// @format
// @ts-check

const chalk = require('chalk');

/**
 * @param {import('../searchers/ripgrep').SearchResult[]} results
 * @return void
 */
function outputResults(results) {
	const messages = results.map(match => {
		return `${chalk.default.magenta(match.path)}:${chalk.default.green(match.line)}:${match.text}`;
	});
	messages.map(message => console.log(message));
}

module.exports = outputResults;
