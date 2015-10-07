Exprolution
=============

This is a toy example that uses a genetic algorithm to find an expression that
evaluates to a given number. For example, if the target is `42`, an example
solution would be
    
    7*7-7

Note that there are infinitely many solutions to each instance of this problem.
However, we just care about finding one at this point (as opposed to the
shortest non trivial solution).

The original idea is from the excellent [tutorial on genetic algorithms found on
AI Junkie][1].


###Running

1. [Install][2] the nightly version of Rust.
2. Clone this repo and go to the cloned directory:
    
        $ git clone https://github.com/yati-sagade/exprolution.git
        $ cd exprolution

3. Do
    
        $ cargo run <some-number>

4. Goto #3 


### Example runs
    $ cargo run 17
         Running `target/debug/exprolution 17`
    Found a solution in 3 generations:
        17-1*0

    $ cargo run 42
         Running `target/debug/exprolution 42`
    Found a solution in 2 generations:
        072-22*1**810+8

    $ cargo run 271828
         Running `target/debug/exprolution 271828`
    Found a solution in 8 generations:
        6*45235+418


### Notes
- In the current expression evaluation scheme, while the operators have
correct relative precedence, the associativity for equal precedence operators
is right-to-left (as opposed to left-to-right, which is how most programming
languages have it). So, the expression `1 / 2 / 3` is evaluated as `(1 / (2 / 3))`
and not as `((1 / 2) / 3)`.

- Sometimes, the algorithm comes up with "cheat solutions", e.g., when you ask
for an expression that evaluates to `12345`, it reports the "expression" as
`12345`. If that happens, just retry :)

- exprolution = expression + evolution



[1]: http://www.ai-junkie.com/ga/intro/gat1.html
[2]: https://www.rust-lang.org/downloads.html
