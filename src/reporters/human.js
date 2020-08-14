// @ts-check

const chalk = require('chalk').default;

/**
 * @param {import('../searchers/ripgrep').SearchResult[]} results
 * @param {import('../general').SearchConfig} config
 * @return {string[]}
 */
function outputResults(results, { showLineNumbers, disableColor }) {
	const ctx = disableColor ? new chalk.constructor({ level: 0 }) : chalk;
	return results.map(match => {
		if (showLineNumbers) {
			return `${ctx.magenta(match.path)}:${ctx.green(String(match.line))}:${match.text}`;
		}
		return `${ctx.magenta(match.path)}:${match.text}`;
	});
}

module.exports = outputResults;
