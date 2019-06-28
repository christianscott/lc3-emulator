.orig x3000

ld  r0, char
lea r1, string
jsr countChar

ld r0, count
out

lea r0, msg1
puts

ld  r0, char
out

lea r0, msg2
puts

lea r0, string
puts
halt

; countChar
; Counts the occurrences of a character in a string
;
; Modifies R0, R1, R2 and R7
;
; @param   {char}    R0 - the character we are looking for
; @param   {string}  R1 - the string we are searching
; @returns {number}  R3 - the count

countChar
        st r7, charCountReturn

        ; twos comp of r0
        not    r0, r0
        add    r0, r0, #1

        ; clear R2
        and    r3, r3, #0

    loop
        ldr    r2, r1, #0             ; load the current char
        brz    end

        ; check if chars are equal by adding current char and negated char
        add    r3, r0, r2
        brz    incr

    loopEnd
        add    r1, r1, #1             ; increment string pointer
        brnzp  loop

    end
        ld     r7, charCountReturn
        st     r3, count
        ret

    incr
        add    r3, r3, #1             ; increment the count
        lea    r6, loopEnd
        jmp    r6

; initialise vars
msg1            .stringz " occurrences of the letter '"
msg2            .stringz "' in the string "
string          .stringz "load rel address"
char            .fill    x64
space           .fill    x20
count           .fill    #0
charCountReturn .blkw    1
