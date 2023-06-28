# Hardware

Some hardware details about some relevant components of the microarchitecture
that differ from the provided components from the book _Structured Computer Organization_.

## ALU

Vondel's ALU have some optimizations oven the Arithmetic Logic Unit provided
by the book, the main difference is that
the function inputs are 3-bit lenght instead of 2-bit, in other words we have
_2<sup>3</sup>_ possible operations that are:

- Logic And
- Logic Or
- Logic Xor
- Logic Not
- Addition
- Multiplication
- Division
- Remainder

For that we change a little bit the hardware provide a circuit to any given
possible operation. The ALU model can be found below:

![ALU hardware info](https://i.imgur.com/Afpp4on.png)

### Adder

The adder is a simple 32-bit full-adder that uses a combination of sequential
1-bit full-adders that conects to each other by the carry, in other words,
the `Co` of one is the `Ci` of the next adder.

### Multiplier

The multiplier circuit uses a standard
[_single cycle multiplier_](https://en.wikipedia.org/wiki/Binary_multiplier)
to multilpy A by B, for that we need to set the inputs and output width to 32-bit lenght.

![32-bit x 32-bit sigle cycle multiplier diagram](https://i.imgur.com/c7yAAmu.png)

### Divider and Remainder

The divider circuit uses the
[_division algorithm_](https://en.wikipedia.org/wiki/Division_algorithm)
logic implemented on digital logic to divide A by B, the basic idea is:

1. Compare the divisor with the selected bits of the dividend.
    1. If is less, then perform the subtraction, that results on a `Q = 1` (`Q` is the Quotient).
    2. If is greater, do not perform the subtraction (`Q = 0`), go to step 2.
2. Add the next bit of the dividend to the result and shift the divisor to right by 1 bit, go to step 1.
3. Repeat the procedure until all the bits of the dividend are covered.

This circuit uses a Process Unit (PU) to perform the comparison logic part of this algorithm,
this simple circuit is ilustrated below:

![Divider process unit](https://i.imgur.com/oNgSOQ1.png)

Combining a matrix of PU's which each row determine a single Quotient bit we
can buid a simple 32-bit x 32-bit adder.

![32-bit by 32-bit divider](https://i.imgur.com/5mwdFDX.png)

You may notice that in the first row we feed the PU inputs only with B and zeros,
the reason is that in the standard divider circuit the dividend lenght is aways 2 times
the divisor lenght, that way whe fill the 32 MSB's of the pseudo 64-bit input with
zeros, that way we can achieve a 32-bit x 32-bit division.

You can simulate a smaller sample of this circuit [here](https://www.circuitlab.com/editor/#?id=f9s285).

## IMM

The IMM circuit provides pre functionality of write the `IMMEDIATE` field
of the microinstruction in the A and/or the B buses. The circuit is fairly
simple and looks like this:

![IMM circuit diagram](https://i.imgur.com/n0zI4kF.png)

## Others

Other integrated circuits of the microarchitecture like the Logic Unit, O and
High Bit are exactly the same as illustrated in the book.
