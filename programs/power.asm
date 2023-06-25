.data
  x: .byte 2       # Value of X
  n: .byte 5       # Value of n

.text
_start:
    lui t0 <- 2        # t0 = X must be a byte
    lui t1 <- 5        # t1 = n must be a byte
    lui t2 <- 0        # T2 is a counter for the loop

    lui ra <- 1        # ra = 1 will be the result of the multiplication

loop:
    beq t2, t1, done      # If the loop counter equals n, exit the loop
    mul ra <- ra, t0      # Multiply ra by X
    addi t2 <- t2, 1      # Increment the loop counter
    jal loop

done:
  halt
