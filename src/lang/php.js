// @format

function getRegexp(symbol) {
	if (symbol.startsWith('$')) {
		const symbolWithoutDollar = symbol.substring(1);
		return `\\$\\b${symbolWithoutDollar}\\b\\s*=`;
	}
	return `\\b(function|class|trait|interface) ${symbol}\\b`;
}

module.exports = getRegexp;
