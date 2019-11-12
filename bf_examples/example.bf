#/kajiit

:kajiit {
    #use src/print/start
    #use src/print/end

    #/start
        "Kajiit! :)\n"
    #/end
}

:print {
    :start {
        [-]->
    }

    :end {
        +[-<+]-> [.>]
    }
}