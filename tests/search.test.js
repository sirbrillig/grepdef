// @ts-check
const { search } = require('../src/general');

describe.each([
	['queryDb', 1],
	['makeQuery', 3],
	['parseQuery', 5],
	['objectWithFunctionShorthand', 9],
	['shorthandFunction', 10],
])("search('%s')", (symbol, expectedLine) => {
	test(`finds line '${expectedLine}'`, async () => {
		const path = './tests/fixtures/js/db.js';
		const config = {
			type: 'js',
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
