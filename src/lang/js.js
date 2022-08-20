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
	const symbolWithDeclaration = `\\b(${declarationKeywords.map(addTrailingSpace).join('|')})${symbol}\\b`;
	const prototypeDeclaration = `\\bprototype\\.${symbol}\\b`
	const methodShorthand = `\\b${symbol}\\([^)]*\\)\\s*(:[^{]+)?\\{`;
	const propertyLonghand = `\\b${symbol}:\\s*`;
	const typedef = `@typedef\\s(\\{[^}]+\\}\\s)?${symbol}\\b`;
	const regexpParts = [
		symbolWithDeclaration,
		prototypeDeclaration,
		methodShorthand,
		propertyLonghand,
		typedef,
	];
	return `(${[regexpParts.join('|')]})`;
}

function addTrailingSpace(keyword) {
	return keyword + '\\s';
}

module.exports = getRegexp;
