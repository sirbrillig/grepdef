const { normalizeType } = require('../src/general');

describe.each([
	['js', 'js'],
	['javascript', 'js'],
	['typescript', 'js'],
	['typescript.tsx', 'js'],
	['javascript.jsx', 'js'],
	['javascriptreact', 'js'],
	['php', 'php'],
])("normalizeType('%s')", (input, expected) => {
	test(`returns '${expected}'`, () => {
		expect(normalizeType(input)).toEqual(expected);
	});
});
