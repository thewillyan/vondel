.data
  a: .word 777
  b: .word 123
  c: .word 777

.text
main:
  read t0 <- a ; load a into t0, t0 is r1 in uarch
  read t1 <- b ; load b into t1, t1 is r2 in uarch
  read t2 <- c ; load c into t2, t2 is r3 in uarch

csw:
  beq t0, t2, a_equals_c ; if a == c, jump to a_equals_c
  mov t0 <- t2 ; a = c 
  lui ra <- 1 ; ra = 1, ra is r0 in uarch
  jal done ; jump to done

a_equals_c:
  mov t2 <- t1 ; c = b no lui ra<- 0 because ra is always 0

done:
  halt
