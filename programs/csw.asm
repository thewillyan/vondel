.data
  a: .word 777
  b: .word 123
  c: .word 777

.text
main:
  read t0 <- a ; load a into t0
  read t1 <- b ; load b into t1
  read t2 <- c ; load c into t2

csw:
  beq t0, t2, a_equals_c ; if a == c, jump to a_equals_c
  mov t0 <- t2 ; a = c 
  lui ra <- 1 ; ra = 1
  jal done ; jump to done

a_equals_c:
  mov t2 <- t1 ; c = b no lui ra<- 0 because ra is always 0

done:
  halt
