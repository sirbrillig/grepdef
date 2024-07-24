# grepdef

Usage: grepdef [options] &lt;symbol&gt; [path(s)]

The **symbol** is the full string name of a class, function, variable, or similar construct.

The **path** is a relative or absolute file path to a file or a directory or a space-separated series of such paths. You can also use UNIX globs which the shell will turn into paths.

If a search path is not provided, this will search starting from the current directory.

You should use the `--type` option whenever possible, but grepdef will try to guess the type if it is not set.

The output is like using grep, but will only show places where that symbol is defined (no partial matches, variable uses, or function calls). The search uses a regular expression so it is unaware of scope and is far from fullproof, but should be easier and faster than a grep by itself.

> [!IMPORTANT]
> grepdef version 3 is a complete rewrite changing from Node to Rust to make it more efficient and to remove the need to have node and ripgrep installed separately.

## CLI Options

-t, --type &lt;TYPE&gt;

The type is a vim-compatible filetype. One of 'js', 'php', 'rs', or an alias for those strings (eg: 'javascript.jsx'). TypeScript is currently considered part of JavaScript so a type of 'typescript' is equivalent to 'js'.

If the type is not provided, grepdef will try to guess the filetype, but this may be inaccurate.

-n, --line-number

Include the line numbers of matches (equivalent to the grep or rg option).

--no-color

Disable colors in output (color is always disabled if not printing to STDOUT).

-h, --help

Print help and usage text.

-V, --version

Print version information.

## Example

```
$ grepdef --type js -n parseQuery
./test/fixtures/js/db.js:7:function parseQuery() {
```

## Installing

To upgrade from version 1 or 2, first you'll need to uninstall the old version using the following command:

```
npm uninstall -g @sirbrillig/grepdef
```

You can install grepdef with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) by running:

```
cargo install grepdef
```

Alternatively, you can download the executable manually from the [Releases page](https://github.com/sirbrillig/grepdef/releases) and install it however you like.

## Using with editors

- vim: See [vim-grepdef](https://github.com/sirbrillig/vim-grepdef)
- VS Code: See [vscode-grepdef](https://github.com/sirbrillig/vscode-grepdef)

