// @format

function getRegexp(symbol) {
	return `\\b((let\\s|var\\s|const\\s|function\\s|class\\s|prototype\\.)${symbol}\\b|${symbol}\\([^)]*\\)\\s*\\{)`;
}

module.exports = getRegexp;
