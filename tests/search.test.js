// @ts-check
const { search } = require('../src/general');

describe.each([
	['queryDb', 'js', 1],
	['makeQuery', 'js', 3],
	['parseQuery', 'js', 5],
	['objectWithFunctionShorthand', 'js', 9],
	['shorthandFunction', 'js', 10],
	['queryDb', 'php', 2],
	['$makeQuery', 'php', 4],
	['parseQuery', 'php', 6],
	['Foo', 'php', 11],
	['Bar', 'php', 14],
	['Zoom', 'php', 17],
])("search('%s', {type: '%s'})", (symbol, type, expectedLine) => {
	test(`finds line '${expectedLine}'`, async () => {
		const path = getFixtureForType(type);
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
 * @param {import('../src/general').FileType} type
 * @returns {string} path
 */
function getFixtureForType(type) {
	switch (type) {
		case 'js':
			return './tests/fixtures/js/db.js';
		case 'php':
			return './tests/fixtures/php/db.php';
		default:
			throw new Error(`Could not find fixture for type '${type}'`);
	}
}
