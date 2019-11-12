:print { [.>] }
:find_start { +[-<+]- }
:set_start {  
    ; Set to 0
    [-]

    ; Subtract 1 0xFF is the value for a "start"
    - 
}
:print_start { 
    [-]- ; Set the current cell to 0xFF; the marker for the strings start
    >    ; Move 1 to the right to make sure the string doesn't overide the marker
}
:print_end { 
    [-] ; Set the correct terminating character 
    +[-<+]- ; Move back to the 0xFF marker that existed earlier 
    > ; The string starts to the right of the marker
    [.>] ; Print the string until finding the terminator; 0x00 
}

#print_start
    "Hello world!"
#print_end