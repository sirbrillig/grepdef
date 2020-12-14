// @ts-check
const { normalizeType } = require('../src/general');

describe.each([
	['js', 'js'],
	['javascript', 'js'],
	['typescript', 'js'],
	['typescript.tsx', 'js'],
	['typescriptreact', 'js'],
	['javascript.jsx', 'js'],
	['javascriptreact', 'js'],
	['php', 'php'],
	['adjadhaowdjw', null],
])("normalizeType('%s')", (input, expected) => {
	if (expected) {
		test(`returns '${expected}'`, () => {
			expect(normalizeType(input)).toEqual(expected);
		});
	} else {
		test('throws an Error', () => {
			expect(() => normalizeType(input)).toThrowError();
		});
	}
});
