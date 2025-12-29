// SPDX-FileCopyrightText: © 2025 Tenstorrent Inc.
// SPDX-License-Identifier: Apache-2.0

//! Constants and topology tables.
//!
//! This module contains:
//! - Terminal color escape codes (`CmdLineColor`)
//! - The fixed galaxy topology (`UBB_GRAPH_EDGES`)
//! - PCIe bus-id decoding helpers for mapping chips into UBB/ASIC positions
//! - SPI register addresses used for Ethernet port disable programming

use crate::graph::Edges;
use crate::graph::GraphCoords;

/// ANSI color escape codes for terminal output.
pub struct CmdLineColor;
impl CmdLineColor {
    pub const PURPLE: &'static str = "\x1b[35m";
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const ENDC: &'static str = "\x1b[0m";
}

/// Edge list defining the full galaxy topology.
///
/// Assumptions:
/// - No loopback links on UBB trays.
/// - Each edge includes both endpoints and the connected port range on each endpoint.
pub const UBB_GRAPH_EDGES: &[Edges] = &[
    /*
    UBB Asic port layout:

            +-------[7:4]-------+
            |                   |
            |                   |
          [3:0]     ASIC     [15:12]
            |                   |
            |                   |
            +------[11:8]-------+

    ASIC numbers in each UBB:

          UBB 0         UBB 1
        +-------+     +-------+
        | 0 | 4 |  -  | 4 | 0 |
        +-------+     +-------+
        | 1 | 5 |  -  | 5 | 1 |
        +-------+     +-------+
        | 2 | 6 |  -  | 6 | 2 |
        +-------+     +-------+
        | 3 | 7 |  -  | 7 | 3 |
        +-------+     +-------+
          |   |         |   |
          UBB 2         UBB 3
          |   |         |   |
        +-------+     +-------+
        | 3 | 7 |  -  | 7 | 3 |
        +-------+     +-------+
        | 2 | 6 |  -  | 6 | 2 |
        +-------+     +-------+
        | 1 | 5 |  -  | 5 | 1 |
        +-------+     +-------+
        | 0 | 4 |  -  | 4 | 0 |
        +-------+     +-------+
     */
    // UBB 1 internal connections
    // L <-> R connections
    Edges {
        node_1: GraphCoords(0, 0),
        node_2: GraphCoords(0, 4),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(0, 1),
        node_2: GraphCoords(0, 5),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(0, 2),
        node_2: GraphCoords(0, 6),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(0, 3),
        node_2: GraphCoords(0, 7),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    // N <-> S connections Col 1
    Edges {
        node_1: GraphCoords(0, 0),
        node_2: GraphCoords(0, 1),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(0, 1),
        node_2: GraphCoords(0, 2),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(0, 2),
        node_2: GraphCoords(0, 3),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // N <-> S connections Col 2
    Edges {
        node_1: GraphCoords(0, 4),
        node_2: GraphCoords(0, 5),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(0, 5),
        node_2: GraphCoords(0, 6),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(0, 6),
        node_2: GraphCoords(0, 7),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // UBB 2 internal connections
    // L <-> R connections
    Edges {
        node_1: GraphCoords(1, 0),
        node_2: GraphCoords(1, 4),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(1, 1),
        node_2: GraphCoords(1, 5),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(1, 2),
        node_2: GraphCoords(1, 6),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(1, 3),
        node_2: GraphCoords(1, 7),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    // N <-> S connections Col 1
    Edges {
        node_1: GraphCoords(1, 0),
        node_2: GraphCoords(1, 1),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(1, 1),
        node_2: GraphCoords(1, 2),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(1, 2),
        node_2: GraphCoords(1, 3),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // N <-> S connections Col 2
    Edges {
        node_1: GraphCoords(1, 4),
        node_2: GraphCoords(1, 5),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(1, 5),
        node_2: GraphCoords(1, 6),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(1, 6),
        node_2: GraphCoords(1, 7),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // UBB 3 internal connections
    Edges {
        node_1: GraphCoords(2, 0),
        node_2: GraphCoords(2, 4),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(2, 1),
        node_2: GraphCoords(2, 5),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(2, 2),
        node_2: GraphCoords(2, 6),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(2, 3),
        node_2: GraphCoords(2, 7),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    // N <-> S connections Col 1
    Edges {
        node_1: GraphCoords(2, 0),
        node_2: GraphCoords(2, 1),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(2, 1),
        node_2: GraphCoords(2, 2),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(2, 2),
        node_2: GraphCoords(2, 3),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // N <-> S connections Col 2
    Edges {
        node_1: GraphCoords(2, 4),
        node_2: GraphCoords(2, 5),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(2, 5),
        node_2: GraphCoords(2, 6),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(2, 6),
        node_2: GraphCoords(2, 7),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // UBB 4 internal connections
    // L <-> R connections
    Edges {
        node_1: GraphCoords(3, 0),
        node_2: GraphCoords(3, 4),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(3, 1),
        node_2: GraphCoords(3, 5),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(3, 2),
        node_2: GraphCoords(3, 6),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    Edges {
        node_1: GraphCoords(3, 3),
        node_2: GraphCoords(3, 7),
        node_1_port: "[15:12]",
        node_2_port: "[3:0]",
    },
    // N <-> S connections Col 1
    Edges {
        node_1: GraphCoords(3, 0),
        node_2: GraphCoords(3, 1),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(3, 1),
        node_2: GraphCoords(3, 2),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(3, 2),
        node_2: GraphCoords(3, 3),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // N <-> S connections Col 2
    Edges {
        node_1: GraphCoords(3, 4),
        node_2: GraphCoords(3, 5),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(3, 5),
        node_2: GraphCoords(3, 6),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    Edges {
        node_1: GraphCoords(3, 6),
        node_2: GraphCoords(3, 7),
        node_1_port: "[11:8]",
        node_2_port: "[7:4]",
    },
    // UBB 0-1 L to R external connections
    Edges {
        node_1: GraphCoords(0, 4),
        node_2: GraphCoords(1, 4),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    Edges {
        node_1: GraphCoords(0, 5),
        node_2: GraphCoords(1, 5),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    Edges {
        node_1: GraphCoords(0, 6),
        node_2: GraphCoords(1, 6),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    Edges {
        node_1: GraphCoords(0, 7),
        node_2: GraphCoords(1, 7),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    // UBB 2-3 L to R connections
    Edges {
        node_1: GraphCoords(2, 4),
        node_2: GraphCoords(3, 4),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    Edges {
        node_1: GraphCoords(2, 5),
        node_2: GraphCoords(3, 5),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    Edges {
        node_1: GraphCoords(2, 6),
        node_2: GraphCoords(3, 6),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    Edges {
        node_1: GraphCoords(2, 7),
        node_2: GraphCoords(3, 7),
        node_1_port: "[15:12]",
        node_2_port: "[15:12]",
    },
    // UBB 0-2 Up and Down external connections
    Edges {
        node_1: GraphCoords(0, 3),
        node_2: GraphCoords(2, 3),
        node_1_port: "[11:8]",
        node_2_port: "[11:8]",
    },
    Edges {
        node_1: GraphCoords(0, 7),
        node_2: GraphCoords(2, 7),
        node_1_port: "[11:8]",
        node_2_port: "[11:8]",
    },
    // UBB 1-3 Up and Down connections
    Edges {
        node_1: GraphCoords(1, 3),
        node_2: GraphCoords(3, 3),
        node_1_port: "[11:8]",
        node_2_port: "[11:8]",
    },
    Edges {
        node_1: GraphCoords(1, 7),
        node_2: GraphCoords(3, 7),
        node_1_port: "[11:8]",
        node_2_port: "[11:8]",
    },
];

/// Decode the *first* PCIe bus-id nibble into a WH UBB tray index.
pub const fn wh_pcie_to_ubb_tray(c: char) -> Option<usize> {
    match c {
        'c' => Some(0),
        '8' => Some(1),
        '0' => Some(2),
        '4' => Some(3),
        _ => None,
    }
}

/// Decode the *first* PCIe bus-id nibble into a BH UBB tray index.
pub const fn bh_pcie_to_ubb_tray(c: char) -> Option<usize> {
    match c {
        '0' => Some(0),
        '4' => Some(1),
        'c' => Some(2),
        '8' => Some(3),
        _ => None,
    }
}

/// Decode the *second* PCIe bus-id nibble into an ASIC slot index within a tray.
pub const fn pcie_char_to_slot_num(c: char) -> Option<usize> {
    match c {
        '1' => Some(0),
        '2' => Some(1),
        '3' => Some(2),
        '4' => Some(3),
        '5' => Some(4),
        '6' => Some(5),
        '7' => Some(6),
        '8' => Some(7),
        _ => None,
    }
}

/// Base address of the Ethernet parameter table in SPI.
const ETH_PARAM_BASE_ADDR: u32 = 0x21100;

/// Parameter-table register address for Ethernet port disable mask.
pub const ETH_PARAM_PORT_DISABLE: u32 = ETH_PARAM_BASE_ADDR + (0x4) * 2;
