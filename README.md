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
; Semicolons turn the rest of the line into a comment.
; BrainFuck has this thing where anything that's not code
; is automagically a comment. This does not happen here, because
; it makes the error messages worse, which we don't really want.

; A colon followed by an identifier and brackets define a macro
; These can be defined in any order
:test_macro {
    ; Sets the memory at the current pointer to the ascii code for 'b'; then shifts right
    ; This also zeroes the memory; so it's perfectly safe; at the cost of some overheat
    ; I'm planning to add another type of string that doesn't zero the memory; which will
    ; make it faster
    "b" 
    
    ; Move back and print that memory location
    < .
}

; This code copies code from a macro into that spot 
; Paths work like file paths on windows;
; except that the root is called src and not C
; Relative paths exist; but more on those later
#src/test_macro
```
All this program does is output the character ``b``.
As you can see, this is designed to alleviate the annoyance of writing
increments to set strings and duplicate code everywhere, which can happen a lot in bf.

This extension is hence not designed to make bf an easier language, but instead to shift the focus
from boring problems such as hardcoding and code duplication to more interesting problems
like how to work with the limitations of bf to produce interesting results.

```
; Same from previous example
:print_b { "b"<. }

; This is a relative path; It simply checks for this path in this scope 
#/print_b

:sub_scopes {
    ; Scopes can be defined inside of other scopes
    ; Name collisions don't occur in different scopes
    :print_b {
        ; Adding a dot to the beginning of a relative path will move into the parent scope
        ; Adding more dots moves into the parent scope of the parent scope etc
        ; Here we have 2 dots; the first one for moving into sub_scopes and second 
        ; one for going into src 
        #../print_b
    }

    :print_c {
        "c"<.
    }
}

; Demonstration of the directory like structure of macros
#/sub_scopes/print_b

; We can also import a macro into the current scope with the use keyword
; This however does not allow accessing submacros of that macro; which I intend to fix later maybe
#use /sub_scopes/print_c
#/print_c
```
Output: ``bbc``

## Debugging
Placing a '!' in your code
will make it print out the memory at that location. In that way, it's like running
your code with ``*debug`` but only printing the debug information at the '!'.
This command does nothing at all when running the program in debug mode.

```
; A program designed to show the inner workings of the strings
"H" !
"e" !
"l" !
"l" !
"o" !
"!" !
```
Output:

```
instr: ..54, mem: ...1 | DEBUG_DUMP
Memory: 00 00 00 00 00 48 00 00 00 00 00 00 00
                          ^ ...1

instr: ..C6, mem: ...2 | DEBUG_DUMP
Memory: 00 00 00 00 48 65 00 00 00 00 00 00 00
                          ^ ...2

instr: .13F, mem: ...3 | DEBUG_DUMP
Memory: 00 00 00 48 65 6C 00 00 00 00 00 00 00
                          ^ ...3

instr: .1B8, mem: ...4 | DEBUG_DUMP
Memory: 00 00 48 65 6C 6C 00 00 00 00 00 00 00
                          ^ ...4

instr: .234, mem: ...5 | DEBUG_DUMP
Memory: 00 48 65 6C 6C 6F 00 00 00 00 00 00 00
                          ^ ...5

instr: .262, mem: ...6 | DEBUG_DUMP
Memory: 48 65 6C 6C 6F 21 00 00 00 00 00 00 00
                          ^ ...6
```
## Repetitions
It happens quite often that you want to repeat a command or set of commands. To alleviate this problem, you can put a byte formatted in hexadecimal after any command to repeat it that many times.
```
+5 !
>5 !
+10 !
```

Output:
```
instr: ...5, mem: ...0 | DEBUG_DUMP
Memory: 00 00 00 00 00 00 05 00 00 00 00 00 00
                          ^ ...0

instr: ...B, mem: ...5 | DEBUG_DUMP
Memory: 00 05 00 00 00 00 00 00 00 00 00 00 00
                          ^ ...5

instr: ..1C, mem: ...5 | DEBUG_DUMP
Memory: 00 05 00 00 00 00 10 00 00 00 00 00 00
                          ^ ...5
```

It is also quite common to want to increment/decrement a value by the ascii code of a character. Therefore, this is also possible by adding ``'a``, ``'b`` or ``'`` + any other ascii character whose code is the number of repetitions you want that code to run.

```
+'a . >
+'b . >
+'c .
```

Output:
```
abc
```

Last but not least, you might want to repeat several commands a certain number of times. This is also possible, by surrounding the commands you want to repeat with parenthesees, ``()``, + the number of times you want them to repeat.

```
(
    +'a . > 
    +'b . > 
    +'c . >
)5
```

Output:
```
abcabcabcabcabc
```

## STD
There is a small standard library included as well, with some basic functionality for convenience. The std library can be accessed by typing #std followed by the path of the macro you want.

### #std/marker
```
; #std/marker can be used to "mark" a memory location,
; i.e. setting it's value to 0xFF. If you use #std/marker
; you have to make sure that you're not setting anything
; else to 0xFF elsewhere in your program.

#std/marker/set ; Sets a marker at the current location
> "1" ; Put a character to the right of the marker
>>>>>>>>>>>>>> ; Move the marker some amount to the right.

; Searches for a marker
; the left of the memory pointer.
; This will go on for infinity if you're not careful.
#std/marker/find_left

; Check to see if the value to the right of the marker
; is the same character as before
> .

; You can also find the character to the right.
<<<<<<<<<<<<<<<<<<<<<
#std/marker/find_right
> .

; If you want, you can also set all bytes up until the
; marker to 0x00, both to the left and to the right
>>>>>>>>>>>>>>>>
#std/marker/clear_left ; clear_right also exists
> . ; Won't print anything, since the code 0x00 doesn't
    ; Correspond to a character.
```
Output:
```
11
```

### #std/print
```
; In order to make it easier to print things,
; there are some utility macros to help you with that.

; This macro sets a marker and moves the memptr one
; step to the right.
#std/print/start
    "Hello world!\n"
    "This text,\n"
    "is very easy to write, isn't it?\n"

; This macro finds the marker to the left, moves
; to the right and prints all characters until
; it finds a memory location with 0x00, i.e.
; a memory location that doesn't contain a string.
; It leaves the memory ptr just to the right of the
; string data.
#std/print/end

; If you want to clean up the memory after a print,
; you can use #std/print/cleanup. This macro doesn't
; even leave behind a marker, so no memory is left
; from the string
#std/print/cleanup

; If you only need to print from the current memptr
; location until the end of the string it's in, you can
; use #std/print
"Not this, Hi"
<<
#std/print
```
Output:
```
Hello world!
This text,
is very easy to write, isn't it?
Hi
```
