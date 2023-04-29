# email-parser

The fastest and lightest email parsing Rust library!

## History

Started in mid-2020, this library was originally designed to be a handmade, zero-dependency email parsing library.
The performance of this library was unrivalled.
However, the maintainance cost was too high.

I started wondering if I could use Rust macros to generate the parsing code.
I liked [pest](https://pest.rs/) but it had terrible performance.
Using the knowledge I got writing the first email parser, I wrote [a better code generator](https://github.com/Mubelotix/faster-pest) for pest grammars.

Then, I made the current version of this library, whose code is almost **entirely automatically generated**.
The generated code is so overly optimized, it has beaten my older handmade parser.
This is the **fastest email parser** among all those I have tested.
