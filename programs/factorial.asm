.text
_start:
  lui a0, 5        ; Number for which factorial is calculated
  lui a1, 1        ; Initialize a1 to 1 (used as a counter)
  lui ra, 1        ; Initialize ra to 1 (used to multiply)

loop:
  beq a1, a0, done   ; If counter equals N, exit the loop
  mul ra <-  ra, a1  ; Multiply ra by a1
  addi a1, a1, 1     ; Increment the counter
  j loop             ; Jump to the loop

done:
