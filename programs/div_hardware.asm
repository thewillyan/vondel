.data
  N: .word 420
  X: .word 69

.text
main:
  read ra <- N ; ra is the dividend and remainder after the loop on the program it will be r0
  read t1 <- X ; t1 is the divisor it's r2 on program
  div ra <- ra, t1 ; ra is the quotient and r1 on program
  mod t0 <- ra, t1 ; t0 is the remainder and r0 on program

done:
  halt
