<img title="Nexsys Logo" alt="Nexsys Logo" src="./NexsysLogoFull.png">
<br>

**Simple code for complex engineering** 
<br>
Nexsys is a "language" for engineers to help solve equations and develop complex mathematical models to ease the design process.
<br>
The Nexsys crate serves multiple functions and aims to be adaptable to a range of situations. Whether you're solving equations in another application or just trying to crunch out an engineering problem, Nexsys aims to have a solution.
<br>
<br>

# **The `nexsys` Rust crate:**
The nexsys crate offers a broad range of tools for addressing engineering problems. From bare-bones rust implementations of Newton's method to an equation solving engine to a full "interpreter", the nexsys crate offers a number of tools accessible in rust to aid with any engineering problem.
<br>
<br>

# **The `nexsys` Python package:**
While narrower in scope, the nexsys Python package leverages `pyo3` to bring the speed of the Nexsys interpreter and solver engine to the ease-of-use of Python. Use the `Nexsys` solver engine object to programmatically solve systems of equations or the `py_solve` function to interpret Nexsys code in Python and use the results elsewhere. Just run `pip intall nexsys` to get started.
<br>
<br>

# **The `nxc` "compiler":**
For those who just want to solve equations, the nxc compiler offers a cli tool to crunch equations and print solutions in markdown format. Just fire up your equation solver of choice and run 
<br>
`nxc ./yourfile.nxs`
<br>
to generate a `.md` with the solution and steps used to acquire it.