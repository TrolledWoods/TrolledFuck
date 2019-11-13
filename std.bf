; STANDARD LIBRARBY

🧙🧙 🧙 🧙 🧙 🧙 🧙 
🧙 🧙 
🧙 🧙 🧙 🧙 🧙 
🧙 THIS IS BLACK MAGIC!! 🧙 
MAGIC

#std/print/start
    "\n\n----STD----\n\nThe std library cannot be exectued\n"
    "Or well, it can, but it isn't really recommended\n"
    "Also, it doesn't do anything!\n"
#std/print/end

; Clear the marker's stuff
#std/marker/clear_left+

:marker {
    :set {
        [-]-
    }

    :clear_left {
        +[-[-]<+]-
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