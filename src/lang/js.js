// @format

const declarationKeywords = [
	'let',
	'var',
	'const',
	'function',
	'class',
	'interface',
	'type',
];

function getRegexp(symbol) {
	const symbolWithDeclaration = `(${declarationKeywords.map(addTrailingSpace).join('|')})${symbol}\\b`;
	const prototypeDeclaration = `prototype\\.${symbol}\\b`
	const methodShorthand = `${symbol}\\([^)]*\\)\\s*(:[^{]+)?\\{`;
	const methodLonghand = `${symbol}:\\s*function\\([^)]*\\)\\s*\\{`;
	const methodArrowFunction = `${symbol}:\\s*\\([^)]*\\)\\s*=>\\s*\\{`;
	const propertyLonghand = `${symbol}:\\s*`;
	const regexpParts = [
		symbolWithDeclaration,
		prototypeDeclaration,
		methodShorthand,
		methodLonghand,
		methodArrowFunction,
		propertyLonghand,
	];
	return `\\b(${[regexpParts.join('|')]})`;
}

function addTrailingSpace(keyword) {
	return keyword + '\\s';
}

module.exports = getRegexp;
