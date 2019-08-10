// @format

function getRegexp(symbol) {
	return `\\b([$]|function )${symbol}\\b`;
}

module.exports = getRegexp;
