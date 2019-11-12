#/print_start 
    "hello world!" 
#/print

:print {
    +[-<+]-> [.>]
}

:print_start_real {
    [-]->
}

:print_start {
    

    :kajiit {
        #src/print_start 
            "Kajiit!" 
        #src/print
    }

    #./print_start_real
}