; #std/marker can be used to "mark" a memory location,
; i.e. setting it's value to 0xFF. If you use #std/marker
; you have to make sure that you're not setting anything
; else to 0xFF elsewhere in your program.

#std/marker/set ; Sets a marker at the current location
> "111111" ; Put a character to the right of the marker
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