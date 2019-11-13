# TrolledFuck
This is a compiler/vm for executing TrolledFuck written in rust, an extension of the isoteric language brainfuck that
implements macros and strings.

## Running code
Compile the program, or use cargo run, with only the path of the file you want to
compiler and execute in order to run it.

``cargo run bf_examples/example.bf`` with shell located in main directory. ``cargo`` has to be installed(obviously).

## Compiler arguments
A compiler argument can be added by writing ``*`` followed by the name of the argument.

* ``*print_bin``; Prints the compiled output as brainfuck, so that you can copy paste it to get bonus swag or to confirm that the program makes sense.
* ``*debug``; Runs the program in debug mode, which makes the compiler print every command it runs.
* ``*bin=[file_path]``; Creates a file at the specified path that containes the compiled binary program. If you run this program(simply by writing ``cargo run file_path_of_binary``(It can autodetect whether it's a binary or not), it can run it immediately and skip the compilation step)
* ``*in=[string]``; Adds the specified string(not surrounded by double quotes, spaces are not supported here) to the program input stream.
* ``*bf``; Compiles the program with the pure rules for brainfuck, and doesn't implement the trolledfuck macros. (Not implemented yet)

## Syntax
```
Anything that isn't an actual program character is a comment
This might change in the future because writing comments like this is unstable and hateful

A colon followed by an identifier and brackets define a macro
These can be defined in any order
:test_macro {
    Sets the memory at the current pointer to the ascii code for 'b'; then shifts right
    This also zeroes the memory; so it's perfectly safe; at the cost of some overheat
    I'm planning to add another type of string that doesn't zero the memory; which will
    make it faster
    "b" 
    
    Move back and print that memory location
    < .
}

This code copies code from a macro into that spot 
Paths work like file paths on windows;
except that the root is called src and not C
Relative paths exist; but more on those later
#src/test_macro
```
All this program does is output the character ``b``.
As you can see, this is designed to alleviate the annoyance of writing
increments to set strings and duplicate code everywhere, which can happen a lot in bf.

This extension is hence not designed to make bf an easier language, but instead to shift the focus
from boring problems such as hardcoding and code duplication to more interesting problems
like how to work with the limitations of bf to produce interesting results.

```
Same from previous example
:print_b { "b"<. }

This is a relative path; It simply checks for this path in this scope 
#/print_b

:sub_scopes {
    Scopes can be defined inside of other scopes
    Name collisions don't occur in different scopes
    :print_b {
        Adding a dot to the beginning of a relative path will move into the parent scope
        Adding more dots moves into the parent scope of the parent scope etc
        Here we have 2 dots; the first one for moving into sub_scopes and second 
        one for going into src 
        #../print_b
    }

    :print_c {
        "c"<.
    }
}

Demonstration of the directory like structure of macros
#/sub_scopes/print_b

We can also import a macro into the current scope with the use keyword
This however does not allow accessing submacros of that macro; which I intend to fix later maybe
#use /sub_scopes/print_c
#/print_c
```
Output: ``bbc``