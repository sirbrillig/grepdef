# grepdef

A cli tool to search for symbol definitions in various programming languages

```
Usage: grepdef --type <type> <symbol> [path]
```

The type is a vim-compatible filetype. One of 'js', 'php', or an alias for those strings (eg: 'javascript.jsx').

The symbol is the full string name of a class, function, variable, or similar construct.

If a search path is not provided, this will search starting from the current directory.

The output is like using grep, but will only show places where that symbol is defined (no partial matches, variable uses, or function calls). The search uses a regular expression so it is unaware of scope and is far from fullproof, but should be easier than a grep by itself.

## Example

```
$ grepdef --type js parseQuery
./test/fixtures/js/db.js:5:function parseQuery() {
```

## Installing

Currently this requires having [ripgrep](https://github.com/BurntSushi/ripgrep) installed. Other searching tools can be configured but I haven't added any yet.

This tool is a node script, so you must have [node](https://nodejs.org/en/) installed.

```
npm install -g @sirbrillig/grepdef
```

## Using with vim

See [vim-grepdef](https://github.com/sirbrillig/vim-grepdef)
