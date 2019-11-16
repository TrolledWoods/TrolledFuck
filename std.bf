; STANDARD LIBRARBY

; 🧙🧙 🧙 🧙 🧙 🧙 🧙 
; 🧙 🧙 
; 🧙 🧙 🧙 🧙 🧙 
; 🧙 THIS IS BLACK MAGIC!! 🧙 
; MAGIC

#std/print/start
    "\n\n----STD----\n\nThe std library cannot be exectued\n"
    "Or well, it can, but it isn't really recommended\n"
    "Also, it doesn't do anything!\n"
#std/print/end
#std/print/cleanup

:marker {
    :set {
        [-]-
    }

    :clear_left {
        +[-[-]<+]-
    }

    :clear_right {
        +[-[-]>+]-
    }

    :find_left {
        +[-<+]-
    }

    :find_right {
        +[->+]-
    }
}

:print {
    :start {
        #std/marker/set
        >
    }

    :end {
        #std/marker/find_left
        >
        [.>]
    }

    :cleanup {
        +[[-]<]
    }

    [.>]
}