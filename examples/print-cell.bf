+++++[>+++<-]>                          // Set cell value to 15

>[-]>[-]+>[-]+<                         // Set n and d to one to start loop
[                                       // Loop on 'n'
    >[-<-                               // On the first loop
        <<[->+>+<<]                     // Copy V into N (and Z)
        >[-<+>]>>                       // Restore V from Z
    ]
    ++++++++++>[-]+>[-]>[-]>[-]<<<<<    // Init for the division by 10
    [->-[>+>>]>[[-<+>]+>+>>]<<<<<]      // full division
    >>-[-<<+>>]                         // store remainder into n
    <[-]++++++++[-<++++++>]             // make it an ASCII digit; clear d
    >>[-<<+>>]                          // move quotient into d
    <<                                  // shuffle; new n is where d was and
                                        //   old n is a digit
    ]                                   // end loop when n is zero
<[.[-]<]                                // Move to were Z should be and
                                        // output the digits till we find Z
<                                       // Back to V

>++++++++++.
