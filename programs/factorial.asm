.text
_start:
  lui a0 <- 5        ; Number for which factorial is calculated
  lui a1, ra <- 1    ; Initialize a1 to 1 (used as a counter)

loop:
  beq a1, a0, done   ; If counter equals N, exit the loop
  mul ra <- ra, a1  ; Multiply ra by a1
  addi a1 <- a1, 1     ; Increment the counter
  jal loop             ; Jump to the loop

done:
  halt
