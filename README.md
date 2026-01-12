# TT-GLX-PARTITION

This tool is used to disable and restore the Ethernet links in Tenstorrent Galaxy systems. This allows users to cleanly partition the Galaxy into easily virtualized and containerized sections.

## Overview

The tool provides various ways to isolate or restore Ethernet connectivity between ASICs in a Tenstorrent Galaxy system:

1. **UBB-level operations**: Isolate or restore entire UBB trays
2. **Config-based operations**: Use TOML config files to define custom groups of ASICs for isolation/restoration

## Important Notes

> [!IMPORTANT]
> **VM/Container Restriction**: **This tool cannot be run in a VM or Docker container**. The PCIe bus IDs detected in virtualized environments may differ from the physical hardware, which will cause incorrect device mapping. The tool requires direct access to the physical PCIe devices.
>
> **Device Detection**: The tool requires exactly 32 chips to be detected. If fewer are detected, it will exit with an error. This helps ensure you're running on the correct hardware and not in a virtualized environment.

> [!NOTE]
> **After running isolation/restore operations**, you must run `tt-smi -glx_reset` to apply the changes to the hardware.

## Installation

If Rust is not installed on your system, please run the following:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Then, clone the repo and run `cargo run -- <ARGS>` to run the program.

## Command Line Options

### UBB-Level Operations

#### Isolate Operations
- `--isolate-all-trays`: Isolate all UBB trays from each other. This disables all inter-UBB Ethernet connections.
- `--isolate-tray <UBB_NUMBER>`: Isolate a specific UBB tray from others. Provide UBB number (1-indexed, where 1-4 are valid).

#### Restore Operations
- `--restore-all`: Restore all UBB trays to default Ethernet settings (all ports enabled).
- `--restore-tray <UBB_NUMBER>`: Restore a specific UBB tray to default Ethernet settings. Provide UBB number (1-indexed).

### Config-Based Operations

#### Generate Config File
- `--gen-config [FILENAME]`: Generate a template TOML config file.
  - If no filename is provided, creates `~/.config/tenstorrent/<date>_<time>_glx_eth_config.toml`. If that fails, then it will create it in local directory under `config`.
  - If a filename is provided, creates the file at the specified path

#### Use Config File
- `--use-config <MODE> <FILENAME>`: Use a TOML config file to specify groups of ASICs.
  - **MODE**: Either `isolate` or `restore`
    - `isolate`: Isolate groups of ASICs based on the config file using the graph algorithm
    - `restore`: Restore groups of ASICs to default settings based on the config file
  - **FILENAME**: Path to the TOML config file

### Other Options
- `--gui`: Launch the GUI interface (work in progress - currently just a template)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Config File Format

The TOML config file defines groups of ASICs that should be isolated together. Each group must form a connected component in the Galaxy topology. 

Refer to the following Galaxy ASIC layout map when deciding the groups to isolate:

```text
  UBB 1         UBB 2
+-------+     +-------+
| 1 | 5 |  -  | 5 | 1 |
+-------+     +-------+
| 2 | 6 |  -  | 6 | 2 |
+-------+     +-------+
| 3 | 7 |  -  | 7 | 3 |
+-------+     +-------+
| 4 | 8 |  -  | 8 | 4 |
+-------+     +-------+
  |   |         |   |
  UBB 3         UBB 4
  |   |         |   |
+-------+     +-------+
| 4 | 8 |  -  | 8 | 4 |
+-------+     +-------+
| 3 | 7 |  -  | 7 | 3 |
+-------+     +-------+
| 2 | 6 |  -  | 6 | 2 |
+-------+     +-------+
| 1 | 5 |  -  | 5 | 1 |
+-------+     +-------+
```


Example `config.toml`:
```toml
system = "ubb-11"
date = "2024-01-15"
time = "14:30:00"

[[groups]]
group = 1
devices = [
    { ubb_num = 1, asic_num = 1 },
    { ubb_num = 1, asic_num = 2 },
    { ubb_num = 1, asic_num = 5 },
    { ubb_num = 1, asic_num = 6 },
    { ubb_num = 1, asic_num = 7 },
    { ubb_num = 1, asic_num = 8 },
]

[[groups]]
group = 2
devices = [
    { ubb_num = 2, asic_num = 1 },
    { ubb_num = 2, asic_num = 2 },
]
```

**Important Notes:**
- `ubb_num` and `asic_num` are 1-indexed (1-4 for UBB, 1-8 for ASIC)
- Each group must form a connected component (all ASICs in the group must be reachable from each other)
- The tool will validate connectivity before applying isolation

## Usage Examples

### Basic UBB Isolation
```bash
# Isolate all UBB trays from each other
cargo run -- --isolate-all-trays

# Isolate UBB tray 2 from the rest
cargo run -- --isolate-tray 2
```

### Basic UBB Restoration
```bash
# Restore all UBB trays to default settings
cargo run -- --restore-all

# Restore UBB tray 3 to default settings
cargo run -- --restore-tray 3
```

### Config-Based Operations
```bash
# Generate a template config file
cargo run -- --gen-config

# Generate a config file with a specific name
cargo run -- --gen-config my_config.toml

# Isolate ASICs based on config file
cargo run -- --use-config isolate config.toml

# Restore ASICs based on config file
cargo run -- --use-config restore config.toml
```

## Operating Logic

This section explains how the tool models and operates on the Galaxy system topology.

### Graph Layout

```text
UBB layout with chip numbers:

  UBB 1         UBB 2
+-------+     +-------+
| 1 | 5 |  -  | 5 | 1 |
+-------+     +-------+
| 2 | 6 |  -  | 6 | 2 |
+-------+     +-------+
| 3 | 7 |  -  | 7 | 3 |
+-------+     +-------+
| 4 | 8 |  -  | 8 | 4 |
+-------+     +-------+
  |   |         |   |
  UBB 3         UBB 4
  |   |         |   |
+-------+     +-------+
| 4 | 8 |  -  | 8 | 4 |
+-------+     +-------+
| 3 | 7 |  -  | 7 | 3 |
+-------+     +-------+
| 2 | 6 |  -  | 6 | 2 |
+-------+     +-------+
| 1 | 5 |  -  | 5 | 1 |
+-------+     +-------+

Zoomed in chip port layout:

  +-------[7:4]-------+
  |                   |
  |                   |
[3:0]     ASIC     [15:12]
  |                   |
  |                   |
  +------[11:8]-------+

```

The Galaxy system is modeled as an undirected graph representing the Ethernet connectivity between ASICs:

- **Nodes**: Each node represents an ASIC, identified by `(UBB_index, ASIC_index)` coordinates
  - 4 UBB trays (indices 0-3)
  - 8 ASICs per UBB tray (indices 0-7)
  - Total: 32 nodes in the graph

- **Edges**: Each edge represents an Ethernet connection between two ASICs
  - Each edge contains metadata specifying which port ranges on each ASIC form the connection
  - Port ranges are specified as `"[3:0]"`, `"[7:4]"`, `"[11:8]"`, `"[15:12]"` representing the physical Ethernet ports on each ASIC

### Graph Caching

For efficient operations, the graph is cached with optimized data structures:

1. **Adjacency Bitmask**: A 32-element array where each element is a 32-bit integer. Bit `j` in `adjacency_bitmask[i]` is set if node `i` is connected to node `j`. This allows O(1) neighbor checks using bitwise operations.

2. **Node Index Mapping**: A hash map that converts `GraphCoords` (the `(UBB, ASIC)` tuple) to petgraph `NodeIndex` for fast lookups when traversing the graph.

This caching enables fast connectivity checks and neighbor traversal without repeatedly searching the graph structure.

### Connectivity Validation with Union-Find

Before isolating a group of ASICs, the tool validates that they form a single connected component using a Union-Find (Disjoint Set Union) data structure.

**Why this matters**: A disconnected group cannot communicate internally, making isolation meaningless. The Union-Find algorithm efficiently validates connectivity.

### Port Disable Calculation

Once connectivity is validated, the tool calculates which ports must be disabled:

1. For each ASIC in the selected group, examine all its neighbors
2. If a neighbor is **outside** the selected group, the connecting port on the selected ASIC is marked for disabling
3. This ensures the group is isolated from the rest of the system while maintaining internal connectivity

## Getting Help

If you need help or have questions about using this tool:

1. **Check the documentation**: Review this README, especially the [Usage Examples](#usage-examples) and [Config File Format](#config-file-format) sections
2. **Search existing issues**: Your question may already be answered in [closed issues](https://github.com/tenstorrent/tt-glx-partition/issues?q=is%3Aissue)
3. **Ask a question**: Open a new issue using our [question template](https://github.com/tenstorrent/tt-glx-partition/issues/new?template=question.md)

The development team monitors this repository [DEVELOPERS CHECK THIS OFTEN] and will respond to questions and issues as quickly as possible.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on how to contribute to this project.

## Security

If you discover a security vulnerability, please see our [Security Policy](./SECURITY.md) for instructions on how to report it responsibly.

## License

This project is licensed under [Apache License 2.0](./LICENSE). You have permission to use this code under the conditions of the license pursuant to the rights it grants.

This software assists in programming Tenstorrent products. Making, using, or selling hardware, models, or IP may require the license of rights (such as patent rights) from Tenstorrent or others.
