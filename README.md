# caro-ai
AI engine designed to play [Caro](https://en.wikipedia.org/wiki/Gomoku#Caro), a strategic board game and variant of Gomoku. The goal in Caro is to place five stones consecutively in a row, column, or diagonal on a grid. The engine uses the [Monte Carlo Tree Search](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search) algorithm to explore potential moves and their outcomes, using a heuristic that evaluates the number of consecutive stones each player has.  
To enhance efficiency, the engine uses pruning techniques, stopping the exploration of forced moves early, and also uses multiprocessing to parallelize computations, thereby accelerating computation time.
## Usage
Prerequisite: have [cargo installed](https://doc.rust-lang.org/cargo/getting-started/installation.html).  
To play with the engine, run
```sh
cargo run
```
to run the program with terminal ui. Then type coordinates to make your move, the engine would then make a move.
![image](https://github.com/user-attachments/assets/95783775-25ad-4d39-8806-6e2191ff9986)
