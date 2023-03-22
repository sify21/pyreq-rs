# About

A parser for [python requirement files](https://pip.pypa.io/en/stable/reference/requirements-file-format/) using [nom](https://github.com/rust-bakery/nom).

nom includes parsers(functions that take inputs and return IResult), parser generators(general functions that take specific indicating arguments and return a specific parser of one kind), parser combinators(functions that take parsers and return a new combined parser), and some useful testing functions(like `is_alphanumeric`).
