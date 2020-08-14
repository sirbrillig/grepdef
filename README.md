# grepdef

A cli tool to search for symbol definitions in various programming languages

Usage: grepdef [--type <type>] <symbol> [path]

The type is a vim-compatible filetype. One of 'js', 'php', or an alias for those strings (eg: 'javascript.jsx'). Typescript is currently considered part of JavaScript so a type of 'typescript' is equivalent to 'js'.

If the type is not provided, grepdef will try to guess the filetype, but this may be inaccurate.

The symbol is the full string name of a class, function, variable, or similar construct.

The path is a relative or absolute file path to a file or a directory or a space-separated series of such paths. You can also use UNIX globs which the shell will turn into paths.

If a search path is not provided, this will search starting from the current directory.

The output is like using grep, but will only show places where that symbol is defined (no partial matches, variable uses, or function calls). The search uses a regular expression so it is unaware of scope and is far from fullproof, but should be easier than a grep by itself.

## Example

```
$ grepdef --type js parseQuery
./test/fixtures/js/db.js:5:function parseQuery() {
```

## Installing

This tool is a node script, so you must have [node](https://nodejs.org/en/) installed.

```
npm install -g @sirbrillig/grepdef
```

## Using with vim

See [vim-grepdef](https://github.com/sirbrillig/vim-grepdef)

## Acknowledgments

This uses the amazing [ripgrep](https://github.com/BurntSushi/ripgrep) tool for searching and the [vscode-ripgrep](https://github.com/microsoft/vscode-ripgrep) library to install and access it.
