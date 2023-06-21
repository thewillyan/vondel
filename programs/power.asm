.data
  X: .word 2       # Value of X
  n: .word 5       # Value of n

.text
_start:
    lui mar, t3 <- 0   # Set the memory address register and a counter to 0
    read a0, t0      # Read X

    lui mar <- 1       # Set the memory address register to 1
    read a1, t1       # Read n

    lui ra <- 1        # ra = 1

loop:
    beq t3, t1, done    # If the loop counter equals n, exit the loop
    mul ra <- ra, t0      # Multiply ra by X
    addi t3 <- t3, 1      # Increment the loop counter
    jal loop

done:
