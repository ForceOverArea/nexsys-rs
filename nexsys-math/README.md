# **[->] Nexsys**
An equation solving program written in Rust 🦀

Nexsys (i**N**tuitive **E**quations, e**X**pressions, and **SYS**tems) is 
inspired by Engineering Equation Solver (EES), which is a program intended 
for engineers working with thermal or fluids-heavy systems. This tool allows 
engineers to develop massive mathematical models of the systems they work with 
and produce optimized designs in less time than it might take to develop the 
same model in Python or Matlab.

The main drawback of EES is its closed-source nature and poor ability to 
integrate with non-Windows machines and other software, so with an interest in 
simulation and computer-aided engineering, I took it upon myself to fill the gap.
<br>
<br>
# The `nexsys-math` Crate
This is the `nexsys-math` sub-crate, which is responsible for providing basic
from-scratch implementations of useful calculus and linear-algrebra operations (e.g. matrix multiplication, partial derivatives, jacobian matrices). 