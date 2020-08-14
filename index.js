// @ts-check

const minimist = require('minimist');
const { searchAndReport, normalizeType } = require('./src/general.js');

function printHelp() {
	const helpText = `
grepdef: search for symbol definitions in various programming languages

Usage: grepdef --type <type> <symbol> [path]

The type is a vim-compatible filetype. One of 'js', 'php', or an alias for
those strings (eg: 'javascript.jsx'). Typescript is currently considered part
of JavaScript so a type of 'typescript' is equivalent to 'js'.

The symbol is the full string name of a class, function, variable, or similar
construct.

The path is a relative or absolute file path to a file or a directory or a
space-separated series of such paths. You can also use UNIX globs which the
shell will turn into paths.

If a search path is not provided, this will search starting from the current
directory.

The output is like using grep, but will only show places where that symbol is
defined (no partial matches, variable uses, or function calls). The search uses
a regular expression so it is unaware of scope and is far from fullproof, but
should be easier than a grep by itself.
	`;
	console.log(helpText);
}

async function grepdef(args) {
	const options = minimist(args);
	const langType = options.type;
	const searchTool = options.searcher || 'ripgrep';
	const reporterName = options.reporter || 'human';
	const searchSymbol = options._[0];
	const path = options._.slice(1).join(' ') || '.';
	if (options.h || options.help) {
		printHelp();
		process.exit(0);
		return;
	}
	if (!searchSymbol) {
		console.error('No search symbol provided.');
		printHelp();
		process.exit(1);
		return;
	}
	searchAndReport(
		searchSymbol,
		{
			type: normalizeType(langType),
			verbose: !!options.verbose,
			searchTool,
			path,
		},
		reporterName
	).catch(error => {
		console.error(error.message);
		printHelp();
		process.exit(1);
	});
}

module.exports = grepdef;
