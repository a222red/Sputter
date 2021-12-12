# Sputter: A pure-functional language written entirely in Rust
## Features:
- Pure functionality: Every function is a single expression that will produce consistent results given the same input.
- Simple syntax: Sputter's syntax resembles a simpler version of Common Lisp
- Readable errors: Sputter has Clang-style error messages that show the line and token where the error was detected
- Immutability: Variables are evaluated once when they enter scope, and cannot be modified or reevaluated afterward.
- Strict control flow: Every `if` statement must have a corresponding `else` to prevent unintentional returns
- Simple type system: Sputter features a `list` type rather than unevaluated cons-pairs
- Semi-gradual typing: Sputter is dynamically typed, but function parameters can optionally specify a type, and an error will be thrown from the caller rather than the callee if said type isn't matched.
