
; Memory layout: :main menu :intro

; Make it easy to find the start of the program
#set_strong_marker

; MAIN MENU MEMORY
#std/marker/set>
0"-- MAIN MENU --\n"
0"\n"
0"0) Play intro\n"
0"\n[Input]\n"

>>>>

#std/marker/find_left
> #std/print
> ,
> [-] 






:set_strong_marker {
    [-]--
}

:find_strong_marker_left {
    ++[--<++]--
}