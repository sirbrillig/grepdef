// @format

function getRegexp(symbol) {
	return `\\b(let|var|const|function|class) ${symbol}\\b`;
}

module.exports = getRegexp;
