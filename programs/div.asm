.data
  N: .word 420
  X: .word 69

.text
main:
  read ra <- N ; ra is the dividend and remainder after the loop on the program it will be r0
  read t1 <- X ; t1 is the divisor it's r2 on program

loop:
  blt ra, t1, done
  sub ra <- ra , t1
  addi t0 <- t0, 1 ; t0 is the quotient and r1 on program
  jal loop

done:
  halt
