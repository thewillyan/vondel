# Vondel :floppy_disk:

A simple computer architecture, ISA and Interpreter build with Rust.

The name comes from the
[_**Von** Neumann Mo**del**_](https://en.wikipedia.org/wiki/Von_Neumann_architecture).

## Microarchitecture

The basic idea behind this design is to follow some tips from book _Structured
Computer Organization - Andrew S. Tanenbaum_ while implementing a the ability to
perform opposite operations in a given time in order to reduce clock cycles.

For that we will use two [_datapaths_](https://en.wikipedia.org/wiki/Datapath)
with one [_ALU_](https://en.wikipedia.org/wiki/Arithmetic_logic_unit) each,
but a clock of one is the opposite of the other. In other words, the ALU1 has a
clock _α<sub>1</sub>_ which performs one clock cycle on an interval _δ<sub>1</sub>_,
so the ALU2 must have a clock _α<sub>2</sub> = ¬α<sub>1</sub>_
plus a delay of _δ<sub>1</sub>_.

<div align="center">

|      ![Clock relation diagram](https://i.imgur.com/dOZeaWO.png)      |
| :------------------------------------------------------------------: |
|  The relation between the clock _α<sub>1</sub>_ and _α<sub>1</sub>_. |

</div>

The advantage of this method is that we can share the
[_control store_](https://en.wikipedia.org/wiki/Control_storebbjk) and some
registers without big cost in the Misconstruction size, external hardware
components and additional logic steps that may cost some clock cycles. The
way that the components are shared is shown below:

<div align="center">

|               ![Shared components diagram](https://i.imgur.com/WpGOWy0.png)               |
| :---------------------------------------------------------------------------------------: |
| The datapaths share some components such as the Microprogram, RAM and a shared registers.|

</div>

The way that each datapath is structured is not far from the usual three-bus design
plus a [_IFU_](https://en.wikipedia.org/wiki/Instruction_unit), the techniques
involved a rather simple for now as you can see the diagram below:

<div align="center">

| ![Data path diagram](https://i.imgur.com/T79aDlk.png) |
| :---------------------------------------------------: |
| The diagram of a single datapath (click to zoom).     |

</div>

Those two datapaths will become a [_thread_](https://en.wikipedia.org/wiki/Thread_(computing))
in a future design of this Microarchitecture that is planned to have two task parallel threads.
The implementation of this design is provided by the `uarch` module.
