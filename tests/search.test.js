// @ts-check
const { search } = require('../src/general');

describe.each([
	['queryDb', 'js', 'js', 1],
	['makeQuery', 'js', 'js', 3],
	['parseQuery', 'js', 'js', 5],
	['parseQuery', 'js', 'js_2_files', 5],
	['parseQuery', 'js', 'js_glob_files', 5],
	['objectWithFunctionShorthand', 'js', 'js', 9],
	['shorthandFunction', 'js', 'js', 10],
	['shorthandFunction', undefined, 'js', 10], // auto-detect type
	['shorthandFunction', undefined, 'js_2_files', 10], // auto_detect type
	['shorthandFunction', undefined, 'js_glob_files', 10], // auto_detect type
	['queryDb', 'php', 'php', 2],
	['$makeQuery', 'php', 'php', 4],
	['parseQuery', 'php', 'php', 6],
	['Foo', 'php', 'php', 11],
	['Bar', 'php', 'php', 14],
	['Zoom', 'php', 'php', 17],
	['Zoom', undefined, 'php', 17], // auto-detect type
])("search('%s', {type: '%s', path: '%s'})", (symbol, type, fixtureType, expectedLine) => {
	test(`finds line '${expectedLine}'`, async () => {
		const path = getFixtureForType(fixtureType);
		const config = {
			type,
			searchTool: 'ripgrep',
			path,
		};
		const results = await search(symbol, config);
		expect(results.length).toEqual(1);
		const result = results[0];
		expect(result.path).toEqual(path);
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
		case 'js':
			return './tests/fixtures/js/db.js';
		case 'js_2_files':
			return './tests/fixtures/js/db.js ./tests/fixtures/js/other.js';
		case 'js_glob_files':
			return './tests/fixtures/js/*';
		case 'php':
			return './tests/fixtures/php/db.php';
		default:
			throw new Error(`Could not find fixture for type '${type}'`);
	}
}
