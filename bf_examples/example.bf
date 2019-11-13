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