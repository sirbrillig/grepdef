// @ts-check
const { getReporterForReporterName } = require('../src/general');

describe.each([
	[
		'human',
		{ showLineNumbers: false, disableColor: true },
		[{ path: 'path/to/file.js', line: 4, text: 'function helloWorld() {' }],
		['path/to/file.js:function helloWorld() {'],
	],
	[
		'human',
		{ showLineNumbers: true, disableColor: true },
		[{ path: 'path/to/file.js', line: 4, text: 'function helloWorld() {' }],
		['path/to/file.js:4:function helloWorld() {'],
	],
])("getReporterForReporterName('%s', %s)", (reporterName, config, searchResults, expected) => {
	test(`returns '${expected}'`, () => {
		expect(getReporterForReporterName(reporterName)(searchResults, config)).toEqual(expected);
	});
});
