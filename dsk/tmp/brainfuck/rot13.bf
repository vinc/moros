-,+[                         Read first char and start outer char reading loop
    -[                       Skip forward if character is 0
        >>++++[>++++++++<-]  Set up divisor (32) for division loop
        <+<-[                Set up dividend (x minus 1) and enter division loop
            >+>+>-[>>>]      Inc copy and rem; reduce divisor; skip forward
            <[[>+<-]>>+>]    Special: move rem to divisor and inc quotient
            <<<<<-           Decrement dividend
        ]                    End division loop
    ]>>>[-]+                 End skip loop; zero divisor and reuse for flag
    >--[-[<->+++[-]]]<[      Zero flag unless quotient was 2 or 3; check flag
        ++++++++++++<[       If flag then set up divisor (13) for 2nd division
            >-[>+>>]         Reduce divisor; Normal: increase rem
            >[+[<+>-]>+>>]   Special: inc rem; move to divisor; inc quotient
            <<<<<-           Decrease dividend
        ]                    End division loop
        >>[<+>-]             Add rem back to divisor to get useful 13
        >[                   Skip forward if quotient was 0
            -[               Dec quotient and skip if quotient was 1
                -<<[-]>>     Zero quotient and divisor if quotient was 2
            ]<<[<<->>-]>>    Zero divisor and sub 13 from copy if quotient is 1
        ]<<[<<+>>-]          Zero divisor and add 13 to copy if quotient is 0
    ]                        End outer skip (if ((char minus 1)/32) != 2 or 3)
    <[-]                     Clear rem from first division if 2nd skipped
    <.[-]                    Output ROT13ed character from copy and clear it
    <-,+                     Read next character
]
