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
	const methodShorthand = `${symbol}\\([^)]*\\)\\s*\\{`;
	const regexpParts = [
		symbolWithDeclaration,
		prototypeDeclaration,
		methodShorthand,
	];
	return `\\b(${[regexpParts.join('|')]})`;
}

function addTrailingSpace(keyword) {
	return keyword + '\\s';
}

module.exports = getRegexp;
