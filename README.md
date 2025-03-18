# Sway IPC Communicator for EWW

Written in Rust. Communicates with Sway IPC to allow for `deflisten` for workspaces for [EWW](https://github.com/elkowar/eww).

### Notable Features

- Custom JSON parser (because why not)
- Two connections to IPC to avoid race conditions
- Performance should be good


### Usage

Call with `get-workspaces` argument to return the current workspaces.

Otherwise, the program will print to console whenever a relevant workspace event is detected.

Since EWW can handle JSON arrays, all outputs return in a format `[ "name1", "name2, ...]`. The currently focused window will have "focused" appended to the end of the name (presumably you can easily Regex match it.
