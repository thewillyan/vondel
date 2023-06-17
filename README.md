# Vondel
![Build Status](https://img.shields.io/github/actions/workflow/status/thewillyan/vondel/rust.yml)
![Issues](https://img.shields.io/github/issues/thewillyan/vondel)
![Pull requests](https://img.shields.io/github/issues-pr/thewillyan/vondel)
![License](https://img.shields.io/github/license/thewillyan/vondel)

A simple computer architecture, ISA and Interpreter build with Rust.

The name comes from the
[_**Von** Neumann Mo**del**_](https://en.wikipedia.org/wiki/Von_Neumann_architecture).

## Microarchitecture

The basic idea behind this design is to follow some tips from book _Structured
Computer Organization - Andrew S. Tanenbaum_ while implementing a the ability to
perform operations even if the main clock is in a non-edge level in order to
reduce clock cycles.

For that we will use two [_datapaths_](https://en.wikipedia.org/wiki/Datapath)
with one [_ALU_](https://en.wikipedia.org/wiki/Arithmetic_logic_unit) each, but
but the clock trigger of one is the opposite of the other, that is, ALU1 is
falling-edge triggered and ALU2 is rising-edge triggered. Futhermore,
the clock signal received in ALU2 is a function of the clock signal
of ALU1.

Let's say that ALU1 has a clock _α<sub>1</sub>_ and the delayed version of this
signal is _α<sub>1</sub><sup>'</sup>_. So the ALU2 must have a clock
_α<sub>2</sub> = (α<sub>1</sub> ∧ α<sub>1</sub><sup>'</sup>)_ plus a
delay of _δ<sub>1</sub>_ as shown below:

<div align="center">

|      ![Clock relation diagram](https://i.imgur.com/O3SP6L2.png)      |
| :------------------------------------------------------------------: |
|   _The relation between the clock α<sub>1</sub> and α<sub>2</sub>._  |

</div>

In other words, ALU1 will init it's operation cycle on the falling-edge of
_α<sub>1</sub>_ and end the operation on the rising-edge, while ALU2 will
init it's operation cycle on the rising-edge of _α<sub>2</sub>_ and end the
operation on the falling-edge, taking advantage of the main clock (_α<sub>1</sub>_)
even when it is at high level.

The advantage of this method is that we can share the
[_control store_](https://en.wikipedia.org/wiki/Control_storebbjk) and all the
registers without big cost in the Misconstruction size, external hardware
components and additional logic steps that may cost some clock cycles. The
way that the components are shared is shown below:

<div align="center">

|               ![Shared components diagram](https://i.imgur.com/tlHAPgL.png)               |
| :---------------------------------------------------------------------------------------: |
|_The datapaths share some components such as the Microprogram, RAM and a shared registers._|

</div>

The way that each datapath is structured is not far from the usual three-bus design
plus a [_IFU_](https://en.wikipedia.org/wiki/Instruction_unit), the techniques
involved a rather simple for now as you can see the diagram below:

<div align="center">

| ![Data path diagram](https://i.imgur.com/jxiCXOM.png) |
| :---------------------------------------------------: |
| _The diagram of a single datapath (click to zoom)._   |

</div>

Those two datapaths will become a [_thread_](https://en.wikipedia.org/wiki/Thread_(computing))
in a future design of this Microarchitecture that is planned to have two task parallel threads.
The implementation of this design is provided by the
[`uarch`](https://github.com/thewillyan/vondel/tree/uarch_model/src/uarch) module.
