// @format

const util = require('util');
const childProcess = require('child_process');

const exec = util.promisify(childProcess.exec);

async function searchWithRegexp({ regexp, type, path, verbose }) {
	const command = `rg --type ${type} --json '${regexp}' ${path}`;
	try {
		verbose && console.log('command:', command);
		const { stdout } = await exec(command);
		return searchOutputToArray(stdout);
	} catch (error) {
		return searchOutputToArray('');
	}
}

function searchOutputToArray(output) {
	const outputLines = output
		.split(/\n/)
		.filter(line => isProbablyValidJson(line))
		.map(line => JSON.parse(line));
	const matches = outputLines.filter(line => line.type === 'match');
	return matches;
}

function isProbablyValidJson(text) {
	return text.length > 1 && text[0] === '{' && text[text.length - 1] === '}';
}

module.exports = searchWithRegexp;
