// @ts-check
const { search } = require('../src/general');

describe('search', () => {
	it('works', async () => {
		const config = {
			type: 'js',
			searchTool: 'ripgrep',
			path: './tests/fixtures/js/db.js',
		};
		const expected = [
			{
				path: './tests/fixtures/js/db.js',
				line: 3,
				text: 'const makeQuery = () => {};',
			},
		];
		const result = await search('makeQuery', config);
		expect(result).toEqual(expected);
	});
});
