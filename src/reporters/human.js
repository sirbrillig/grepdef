// @ts-check

const chalk = require('chalk');

/**
 * @param {import('../searchers/ripgrep').SearchResult[]} results
 * @param {import('../general').SearchConfig} config
 * @return void
 */
function outputResults(results, {showLineNumbers}) {
	const messages = results.map(match => {
		if (showLineNumbers) {
			return `${chalk.default.magenta(match.path)}:${chalk.default.green(String(match.line))}:${match.text}`;
		}
		return `${chalk.default.magenta(match.path)}:${match.text}`;
	});
	messages.map(message => console.log(message));
}

module.exports = outputResults;
