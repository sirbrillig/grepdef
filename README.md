# grepdef

Usage: grepdef [options] &lt;symbol&gt; [path]

The symbol is the full string name of a class, function, variable, or similar construct.

The path is a relative or absolute file path to a file or a directory or a space-separated series of such paths. You can also use UNIX globs which the shell will turn into paths.

If a search path is not provided, this will search starting from the current directory.

You should use the `--type` option whenever possible, but grepdef will try to guess the type if it is not set.

The output is like using grep, but will only show places where that symbol is defined (no partial matches, variable uses, or function calls). The search uses a regular expression so it is unaware of scope and is far from fullproof, but should be easier and faster than a grep by itself.

## CLI Options

-t, --type &lt;TYPE&gt;

The type is a vim-compatible filetype. One of 'js', 'php', or an alias for those strings (eg: 'javascript.jsx'). Typescript is currently considered part of JavaScript so a type of 'typescript' is equivalent to 'js'.

If the type is not provided, grepdef will try to guess the filetype, but this may be inaccurate.

-n, --line-number

Show line numbers (1-based).

--searcher &lt;SEARCHER&gt;

Use the specified searcher. Currently only 'ripgrep' is supported.

--reporter &lt;REPORTER&gt;

Use the specified reporter. Currently only 'human' is supported.

--no-color

Disable colors in reporters that support them.

-h, --help

Print this help text.

-v, --version

Print the version information.

## Example

```
$ grepdef --type js parseQuery
./test/fixtures/js/db.js:function parseQuery() {
```

## Installing

This tool is a node script, so you must have [node](https://nodejs.org/en/) installed.

```
npm install -g @sirbrillig/grepdef
```

## Using with editors

- vim: See [vim-grepdef](https://github.com/sirbrillig/vim-grepdef)
- VS Code: See [vscode-grepdef](https://github.com/sirbrillig/vscode-grepdef)


## Acknowledgments

This uses the amazing [ripgrep](https://github.com/BurntSushi/ripgrep) tool for searching and the [vscode-ripgrep](https://github.com/microsoft/vscode-ripgrep) library to install and access it.
