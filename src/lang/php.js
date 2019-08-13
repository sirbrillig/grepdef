// @format

function getRegexp(symbol) {
	if (symbol.startsWith('$')) {
		const symbolWithoutDollar = symbol.substring(1);
		return `\\$\\b${symbolWithoutDollar}\\b\\s*=`;
	}
	return `\\b(function|class) ${symbol}\\b`;
}

module.exports = getRegexp;
