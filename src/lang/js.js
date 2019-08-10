// @format

function getRegexp(symbol) {
	return `\\b(let|var|const|function) ${symbol}\\b`;
}

module.exports = getRegexp;
