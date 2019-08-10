// @format

function getRegexp(symbol) {
	if (symbol.startsWith('$')) {
		const symbolWithoutDollar = symbol.substring(1);
		return `\\$\\b${symbolWithoutDollar}\\b`;
	}
	return `\\bfunction ${symbol}\\b`;
}

module.exports = getRegexp;
