// @ts-check
const { search } = require('../src/general');

describe.each([
	['queryDb', 'js', 'js', 1],
	['queryDbTS', 'js', 'ts', 1],
	['makeQuery', 'js', 'js', 4],
	['makeQueryTS', 'js', 'ts', 4],
	['parseQuery', 'js', 'js', 7],
	['parseQuery', 'js', 'js_2_files', 7],
	['parseQuery', 'js', 'js_directory', 7],
	['parseQueryTS', 'js', 'ts', 7],
	['objectWithFunctionShorthand', 'js', 'js', 15],
	['objectWithFunctionShorthandTS', 'js', 'ts', 15],
	['shorthandFunction', 'js', 'js', 16],
	['shorthandFunctionTS', 'js', 'ts', 16],
	['shorthandFunction', undefined, 'js', 16], // auto-detect type
	['shorthandFunctionTS', undefined, 'ts', 16], // auto-detect type
	['shorthandFunction', undefined, 'js_2_files', 16], // auto_detect type
	['shorthandFunction', undefined, 'js_directory', 16], // auto_detect type
	['shorthandFunction', undefined, 'js_parent_directory', 16], // auto_detect type
	['longhandFunction', undefined, 'js_parent_directory', 25], // auto_detect type
	['longhandFunctionTS', undefined, 'ts', 25], // auto_detect type
	['longhandArrowFunction', undefined, 'js_parent_directory', 34], // auto_detect type
	['longhandArrowFunctionTS', undefined, 'ts', 34], // auto_detect type
	['longhandProperty', undefined, 'js_parent_directory', 43], // auto_detect type
	['longhandPropertyTS', undefined, 'ts', 43], // auto_detect type
	['AnInterface', undefined, 'ts', 59], // auto_detect type
	['AType', undefined, 'ts', 63], // auto_detect type
	['TypeDefObject', undefined, 'ts', 66], // auto_detect type
	['queryDb', 'php', 'php', 2],
	['$makeQuery', 'php', 'php', 4],
	['parseQuery', 'php', 'php', 6],
	['Foo', 'php', 'php', 11],
	['Bar', 'php', 'php', 14],
	['Zoom', 'php', 'php', 17],
	['Zoom', undefined, 'php', 17], // auto-detect type
])("search('%s', {type: '%s', path: '%s'})",

	/**
	 * @param {string} symbol
	 * @param {import('../src/general').FileType} type
	 * @param {string} fixtureType
	 * @param {number} expectedLine
	 */
	(symbol, type, fixtureType, expectedLine) => {
	test(`finds line '${expectedLine}'`, async () => {
		const path = getFixtureForType(fixtureType);

		/** @type {import('../src/general').SearchConfig} */
		const config = {
			type,
			searchTool: 'ripgrep',
			path,
		};

		const results = await search(symbol, config);
		expect(results.length).toEqual(1);
		const result = results[0];
		expect(result.path).toEqual(getExpectedResultPath(fixtureType));
		expect(result.line).toEqual(expectedLine);
		expect(result.text).toBeDefined();
	});
});

/**
 * @param {string} type
 * @returns {string} path
 */
function getFixtureForType(type) {
	switch (type) {
		case 'ts':
			return './tests/fixtures/js/misc.ts';
		case 'js':
			return './tests/fixtures/js/db.js';
		case 'js_2_files':
			return './tests/fixtures/js/db.js ./tests/fixtures/js/other.js';
		case 'js_directory':
			return './tests/fixtures/js';
		case 'js_parent_directory':
			return './tests/fixtures/js-parent';
		case 'php':
			return './tests/fixtures/php/db.php';
		default:
			throw new Error(`Could not find fixture for type '${type}'`);
	}
}

/**
 * @param {string} type
 * @returns {string} path
 */
function getExpectedResultPath(type) {
	switch (type) {
		case 'ts':
			return './tests/fixtures/js/misc.ts';
		case 'js':
		case 'js_2_files':
		case 'js_directory':
			return './tests/fixtures/js/db.js';
		case 'js_parent_directory':
			return './tests/fixtures/js-parent/src/db.js';
		case 'php':
			return './tests/fixtures/php/db.php';
		default:
			throw new Error(`Could not find expected result file for type '${type}'`);
	}
}
