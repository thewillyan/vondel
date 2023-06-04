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
clock $\alpha_1$ which performs one clock cycle on an interval $\delta_1$, so
the ALU2 must have a clock $\delta_2 = \neg \delta_1$ plus a delay of $\delta_1$.

| ![Clock relation diagram](https://i.imgur.com/dOZeaWO.png) |
| :--------------------------------------------------------: |
|  The relation between the clock $\alpha_1$ and $\alpha_2$. |

The advantage of this method is that we can share the
[_control store_](https://en.wikipedia.org/wiki/Control_storebbjk) and some
registers without big cost in the Misconstruction size, external hardware
components and additional logic steps that may cost some clock cycles. The
way that the components are shared is shown below:

Those two datapaths will become a [_thread_](https://en.wikipedia.org/wiki/Thread_(computing))
in a future design of this Microarchitecture that is planned to have two task parallel threads.

|               ![Shared components diagram](https://i.imgur.com/WpGOWy0.png)               |
| :---------------------------------------------------------------------------------------: |
| The datapaths share some components such as the Microprogram, RAM and a shared registers.|

The way that each datapath is structured is not far from the usual three-bus design
plus a [_IFU_](https://en.wikipedia.org/wiki/Instruction_unit), the techniques
involved a rather simple for now as you can see the diagram below:

| ![Data path diagram](https://i.imgur.com/zmysFrc.png) |
| :---------------------------------------------------: |
|           The diagram of a single datapath.           |
