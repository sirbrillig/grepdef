// @format

const chalk = require('chalk');

function outputResults(results) {
	const messages = results.map(match => {
		const { data } = match;
		const { path, lines, line_number } = data;
		return `${chalk.magenta(path.text)}:${chalk.green(line_number)}:${lines.text.trim()}`;
	});
	messages.map(message => console.log(message));
}

module.exports = outputResults;
