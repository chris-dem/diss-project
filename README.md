# Pure Circuit Visualiser

Visualisation tool for the __PureCircuit__ problem.
The tool was built using `Rust` with `Bevy`
as the UI library.

## Running instruction

Rust version used to compile

```bash
cargo 1.85.0 (d73d2caf9 2024-12-31)
```

To download all the necessary libraries and build the project

```rust
cargo build
```

To run the tests run the command

```bash
cargo test
```

To run on release mode

```bash
cargo run --release
```

## Tooling used

## Workspaces

### `macro-export` & `misc-lib`

Workspaces to define the `EnumCycle`
trait and the procedural macro that
auto implements it.

### `pure-circuit-lib` library

*PureCircuit* library that exports the following types:

#### Primitive types

* `Value`: Enum value that contains `Zero, Bot, One`
* `Gate`: Enum gates that contains `Copy, Not, And, Or, Nand, Nor, Purify`

#### `PureCircuit` Graph

The `PureCircuitGraph` data structure. Contains a directed
graph datastructue that contains two types of nodes:
`Value` nodes and `Gate` nodes. Node is equipped
with its value as well as additional information. Moreover,
gate nodes are attached with status nodes that indicate
whether they are `Valid`, `InvalidArity` or `InvalidValue`.
We implement the following core operations:

* `Add` or `Remove` node/edge
* `Update` the gate status
* Get node neighbours
We expand more on its API on files `pure-circuit-lib::graph`

#### Solution Finders/Enumerator

* `backtracking`: Backtracking algorithm implementation
* `evo_search`: Meta-heuristic algorithm implementations
* `base_finder`: PureCircuit API extension for fitness evaluation
* `gate_backtrack`: Set simplification implementation and testing
* `solver_trait`: Trait for the solution finders

### `main-app` library

Main UI implementation. Library used `Bevy`.
We refer to the bevy documentation for more information how it works.
List of components:

