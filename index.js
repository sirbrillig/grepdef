// @ts-check

const minimist = require('minimist');
const { searchAndReport, normalizeType } = require('./src/general.js');
var packageJson = require('./package.json');

function printVersion() {
	console.log(`grepdef ${packageJson.version}`);
}

function printHelp() {
	const helpText = `
grepdef: search for symbol definitions in various programming languages

Usage: grepdef [options] <symbol> [path]

The symbol is the full string name of a class, function, variable, or similar
construct.

The path is a relative or absolute file path to a file or a directory or a
space-separated series of such paths. You can also use UNIX globs which the
shell will turn into paths.

If a search path is not provided, this will search starting from the current
directory.

You should use the '--type' option whenever possible, but grepdef will try to
guess the type if it is not set.

The output is like using grep, but will only show places where that symbol is
defined (no partial matches, variable uses, or function calls). The search uses
a regular expression so it is unaware of scope and is far from fullproof, but
should be easier and faster than a grep by itself.

Options:

-t, --type <TYPE>

  The type is a vim-compatible filetype. One of 'js', 'php', or an alias for
  those strings (eg: 'javascript.jsx'). Typescript is currently considered part
  of JavaScript so a type of 'typescript' is equivalent to 'js'.

  If the type is not provided, grepdef will try to guess the filetype, but this
  may be inaccurate.

-n, --line-number

  Show line numbers (1-based).

--searcher <SEARCHER>

  Use the specified searcher. Currently only 'ripgrep' is supported.

--reporter <REPORTER>

  Use the specified reporter. Currently only 'human' is supported.

--no-color

  Disable colors in reporters that support them.

-h, --help

  Print this help text.

-v, --version

  Print the version information.
`;
	console.log(helpText);
}

async function grepdef(args) {
	const options = minimist(args, {
		string: ['type', 'searcher', 'reporter'],
		boolean: ['n', 'line-number', 'help', 'h', 'no-color', 'v', 'version'],
	});
	const langType = options.type;
	const searchTool = options.searcher || 'ripgrep';
	const reporterName = options.reporter || 'human';
	const showLineNumbers = options.n || options['line-number'];
	const disableColor = options['no-color'];
	const searchSymbol = options._[0];
	const path = options._.slice(1).join(' ') || '.';
	if (options.h || options.help) {
		printHelp();
		process.exit(0);
	}
	if (options.v || options.version) {
		printVersion();
		process.exit(0);
	}
	if (!searchSymbol) {
		console.error('No search symbol provided.');
		printHelp();
		process.exit(1);
	}
	try {
		await searchAndReport(
			searchSymbol,
			{
				type: normalizeType(langType),
				verbose: !!options.verbose,
				searchTool,
				path,
				showLineNumbers,
				disableColor,
			},
			reporterName
		);
	} catch (error) {
		console.error(error.message);
		console.error('Try running "grepdef --help" for information about how to use this tool.');
		process.exit(1);
	}
}

module.exports = grepdef;
