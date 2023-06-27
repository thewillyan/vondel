.data
  x: .byte 2       # Value of X
  n: .byte 5       # Value of n

.text
_start:
    lui a0 <- x        # a0 = X must be a byte
    lui a1 <- n        # a1 = n must be a byte
    lui a2 <- 0        # a2 is a counter for the loop

    lui ra <- 1        # ra = 1 will be the result of the multiplication, ra will be r0 in microarchitecture

loop:
    beq a2, a1, done      # If the loop counter equals n, exit the loop
    mul ra <- ra, a0      # Multiply ra by X
    addi a2 <- a2, 1      # Increment the loop counter
    jal loop

done:
  halt
