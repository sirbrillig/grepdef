// @format
// @ts-check

const util = require('util');
const childProcess = require('child_process');
const { rgPath } = require('vscode-ripgrep');

const exec = util.promisify(childProcess.exec);

/**
 * @typedef {import('../general').SearchResult} SearchResult
 */

/**
 * @param {string} regexp - the regexp to search for
 * @param {import('../general').SearchConfig} config
 * @returns {Promise<SearchResult[]>}
 */
async function searchWithRegexp(regexp, { type, path, verbose }) {
	const fileTypes = ['js', 'ts'].includes(type) ? ['js', 'ts'] : [type];
	const typeOptions = fileTypes.map(fileType => `--type ${fileType}`).join(' ');
	const command = `${rgPath} ${typeOptions} --json '${regexp}' ${path}`;
	try {
		verbose && console.log('command:', command);
		const { stdout } = await exec(command);
		return searchOutputToArray(stdout);
	} catch (error) {
		return searchOutputToArray('');
	}
}

/**
 * @param {string} output
 * @returns {SearchResult[]}
 */
function searchOutputToArray(output) {
	return output
		.split(/\n/)
		.filter(line => isProbablyValidJson(line))
		.map(line => JSON.parse(line))
		.filter(line => line.type === 'match')
		.map(convertRipGrepToSearchResult);
}

/**
 * @param {object} ripgrepResult
 * @returns {SearchResult}
 */
function convertRipGrepToSearchResult(ripgrepResult) {
	return {
		path: ripgrepResult.data.path.text,
		line: ripgrepResult.data.line_number,
		text: ripgrepResult.data.lines.text.trim(),
	};
}

/**
 * @param {string} text
 * @returns {boolean}
 */
function isProbablyValidJson(text) {
	return text.length > 1 && text[0] === '{' && text[text.length - 1] === '}';
}

module.exports = searchWithRegexp;
