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

Since we are using `Bevy`, additional
installations may be needed such as build tools.
Please refer to the website
[https://bevy.org/learn/quick-start/getting-started/setup/](https://bevy.org/learn/quick-start/getting-started/setup/)
for further installation instructions, in case you are facing GPU issues.  

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
  * Implementation of the Hill Climbing and the Genetic Algorithm
* `base_finder`: `PureCircuit` API extension for fitness evaluation
* `gate_backtrack`: Set simplification implementation and testing
* `solver_trait`: Trait for the solution finders

### `main-app` library

Main UI implementation. Library used `Bevy`.
We refer to the bevy documentation for more information how it works.
List of important files:

* `ui_plugin`: File for the setup and rendering of the `UI` window
* `main`: Main file that connects all plugins and components. Responsible for running the application.
* `drawing_plugin`: Responsible for the rendering and interaction between graph nodes
* `camera_plugin`: Spawns the camera and handles camera movement such as panning and moving around
* `assets`: Contain the SVG paths for rendering the value nodes
* `state_management`
  * `mouse_state`: Contains the states for mouse management such as `node/edge` modes. Moreover, addition of current mouse position resource
  * `state_init`: Responsible for initialising all essential resources and states of the application.
  * `edge_management`: Responsible initialising the states and handling the edge addition/removal operations.
  * `events`: Initialisation of all events of the application. Set of events that we use:
    * `NodeUpdate`: Re-render the current node and notify the status of all localised gates
    * `NodeStatusUpdate`: Update the status of the gate by re-rendering or removing error circles
    * `ButtonEvoEvent`: Run the `Genetic Algorithm` algorithm
    * `ButtonHillEvent`: Run the `Hill Climbing` algorithm
    * `BacktrackEvent`: Run the `Backtracking` algorithm
    * `SolutionReset`: When the current topology of the circuit changes, reset the solution set
    * `IndexReset`: When the current state of the circuit changes, reset the selected solution index
* `algo_execution`
  * `back`: Responsible for importing and running the backtracking algorithm
  * `plugin`: Responsible for importing and running the meta-heuristic algorithms

## Test outputs

We provide a full exert of our test results in `test-output.log`
