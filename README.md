# Tokay

![Tokay Logo](assets/tokay.svg)

An imperative, procedural programming language dedicated to parsing and other text-processing tasks.

## About

Tokay is a programming language designed for ad-hoc parsing. It is heavily inspired by [awk](https://en.wikipedia.org/wiki/AWK), but follows its own philosophy and design principles. It might also be useful as a general purpose scripting language, but mainly focuses on processing textual input and work on trees with information extracted from this input.

The language was designed to quickly create solutions in text processing problems, which can be just simple pattern matching but even bigger things. Therefore Tokay provides both a language for simple one-liners but also facilites to create programs like code-analysis and refactoring tools, including interpreters or compilers. For example, Tokay's own language parser is implemented using Tokay itself.

Tokay is a very young project and gains much potential. [Volunteers are welcome!](#contribute)

## Highlights

- Concise and easy to learn syntax
- Stream-based input processing
- Automatic parse tree synthesis
- Left-recursive parsing structures ("parselets") supported
- Implements a memoizing packrat parsing algorithm internally
- Robust due to its implementation in only safe [Rust](https://rust-lang.org)
- Enabling awk-style one-liners in combination with other tools
- Generic functions and parselets (*coming soon)
- Interoperability with other shell commands (*coming soon)

There are plenty of further features planned, see [TODO.md](TODO.md) for details.

## Examples

This is how Tokay greets the world

```tokay
print("Hello World")
```

but Tokay can also greet any wor(l)d coming in, that's

```tokay
Word print("Hello " + $1)
```

With its build-in abstract-syntax tree synthesis, Tokay is designed as a language for directly implementing ad-hoc parsers. The next program directly implements a left-recursive grammar for parsing and evaluating simple mathematical expressions, like `1+2+3` or `7*(8+2)/5`.

```tokay
Factor : @{
    Integer             # built-in 64-bit signed integer token
    '(' Expr ')'
}

Term : @{
    Term '*' Factor     $1 * $3
    Term '/' Factor     $1 / $3
    Factor
}

Expr : @{
    Expr '+' Term       $1 + $3
    Expr '-' Term       $1 - $3
    Term
}

Expr
```

Tokay can also be used for writing programs without any parsing features.
Next one is a recursive attempt for calculating the faculty of a value.

```
faculty : @x {
    if !x return 1
    x * faculty(x - 1)
}

faculty(4)
```

## Contribute

Contributions of any kind, may it be code, documentation, support or advertising are very welcome!

Take a look into the [TODO.md](TODO.md) or watch for `//fixme`- and `//todo`-comments in the source code for open issues and things to be improved.

Feel free to [contact me](https://phorward.info) on any questions, or directly file [an issue here](https://github.com/phorward/tokay/issues).

Tokay is also my first project written in Rust, therefore I'm sure many things inside the code could easily be improved by more experienced Rustaceans out there.

## Logo

The Tokay logo and icon was designed by [Timmytiefkuehl](https://github.com/timmytiefkuehl), many thanks to him!

## License

Tokay is free software under the MIT license.
Please see the LICENSE file for more details.
