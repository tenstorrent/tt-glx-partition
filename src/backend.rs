// SPDX-FileCopyrightText: © 2025 Tenstorrent Inc.
// SPDX-License-Identifier: Apache-2.0

//! Hardware access + flashing helpers.
//!
//! This module contains the lowest-level operations used by the CLI:
//! - Mapping detected PCIe devices into an `(UBB -> ASIC)` table
//! - Flashing ETH port-disable masks to ASICs over SPI
//! - Isolating/restoring UBB trays

use crate::constants::{
    ETH_PARAM_PORT_DISABLE, bh_pcie_to_ubb_tray, pcie_char_to_slot_num,
    wh_pcie_to_ubb_tray,
};
use crate::graph::{EdgeToDisable, GraphCoords};
use color_eyre::eyre::{Error, Result};
use luwen::api::{ChipImpl, chip::Chip};
use owo_colors::OwoColorize;

/// Build an `(UBB -> ASIC)` map from the PCIe bus IDs of detected chips.
///
/// The return value is a 4x8 table: `ubb_map[ubb_idx][asic_idx] -> Option<Chip>`.
pub fn build_ubb_device_map(devices: Vec<Chip>) -> Result<Vec<Vec<Option<Chip>>>, Error> {
    let mut ubb_map = Vec::with_capacity(4);
    for _ in 0..4 {
        let mut inner_vec = Vec::with_capacity(8);
        for _ in 0..8 {
            inner_vec.push(None);
        }
        ubb_map.push(inner_vec);
    }

    for (dev_num, dev) in devices.into_iter().enumerate() {
        if let Some(device_info) = dev.get_device_info().unwrap() {
            let bus = device_info.bus;
            // check if the bus_id has 2 characters after the 0,if not then add a 0 to the left
            let bus_hex = format!("{:x}", bus);
            let bus_hex = if bus_hex.len() == 1 {
                format!("0{}", bus_hex)
            } else {
                bus_hex
            };
            // Put it into the correct UBB slot based on the mapping
            if dev.as_wh().is_some() {
                // Use the WH mapping
                let ubb_slot = wh_pcie_to_ubb_tray(bus_hex.chars().next().unwrap()).unwrap();
                let asic_slot_num = pcie_char_to_slot_num(bus_hex.chars().nth(1).unwrap()).unwrap();
                ubb_map[ubb_slot][asic_slot_num] = Some(dev);
            } else if dev.as_bh().is_some() {
                // Use the BH mapping
                let ubb_slot = bh_pcie_to_ubb_tray(bus_hex.chars().next().unwrap()).unwrap();
                let asic_slot_num = pcie_char_to_slot_num(bus_hex.chars().nth(1).unwrap()).unwrap();
                println!("{}", format!("Mapping BH Device {} (Bus: 0x{}) to UBB Slot {}, ASIC Slot {}", dev_num, bus_hex, ubb_slot, asic_slot_num).green());
                ubb_map[ubb_slot][asic_slot_num] = Some(dev);
            } else {
                println!("{}", format!("Device {} is neither WH nor BH chip, skipping...", dev_num).red());
            }
        }
    }

    Ok(ubb_map)
}

/// Write `data` to `addr` on the given ASIC via SPI.
///
/// Supports both WH and BH chips.
pub fn flash_asic(dev: &Chip, addr: u32, data: &[u8]) -> Result<(), Error> {
    // flash the WH and BH ASICs over SPI
    println!("{}", format!("Flashing ASIC at address {:#x} with data: {}", addr, hex::encode(data)).magenta());
    if let Some(wh) = dev.as_wh() {
        wh.spi_write(addr, data).unwrap();
        println!("{}", format!("Successfully flashed ASIC at address {:#x}", addr).green());
        Ok(())
    } else if let Some(bh) = dev.as_bh() {
        bh.spi_write(addr, data).unwrap();
        println!("{}", format!("Successfully flashed ASIC at address {:#x}", addr).green());
        Ok(())
    } else {
        let error_msg = format!(
            "Device is not a WH or BH chip, cannot flash ASIC at address {:#x}",
            addr
        );
        println!("{}", format!("Error: {}", error_msg).red());
        Err(Error::msg(error_msg))
    }
}

/// Build a 4-byte ETH port-disable mask from per-port booleans.
///
/// The mask is encoded as a `u16` in little-endian, padded to 4 bytes to match the firmware
/// register width expected by `ETH_PARAM_PORT_DISABLE`.
pub fn generate_port_disable_mask(
    disable_3_0: bool,
    disable_7_4: bool,
    disable_11_8: bool,
    disable_15_12: bool,
) -> [u8; 4] {
    let mut port_disable_mask = 0u16;

    if disable_3_0 {
        port_disable_mask |= 0x000F;
    }
    if disable_7_4 {
        port_disable_mask |= 0x00F0;
    }
    if disable_11_8 {
        port_disable_mask |= 0x0F00;
    }
    if disable_15_12 {
        port_disable_mask |= 0xF000;
    }

    let bytes = port_disable_mask.to_le_bytes();
    [bytes[0], bytes[1], 0, 0]
}

/// Convert a per-node set of disabled port ranges into a single port-disable mask.
pub fn convert_edges_to_disable_to_port_disable_mask(
    edge_to_disable: &EdgeToDisable,
) -> Result<[u8; 4], Error> {
    // Convert the edge to disable to a port disable mask
    let mut disable_3_0 = false;
    let mut disable_7_4 = false;
    let mut disable_11_8 = false;
    let mut disable_15_12 = false;

    for port in &edge_to_disable.ports {
        match *port {
            "[3:0]" => disable_3_0 = true,
            "[7:4]" => disable_7_4 = true,
            "[11:8]" => disable_11_8 = true,
            "[15:12]" => disable_15_12 = true,
            _ => return Err(Error::msg(format!("Unknown port: {}", port))),
        }
    }

    Ok(generate_port_disable_mask(
        disable_3_0,
        disable_7_4,
        disable_11_8,
        disable_15_12,
    ))
}

/// Flash all requested per-node port disables into hardware.
pub fn flash_edges_to_disable(
    edges_to_disable: Vec<EdgeToDisable>,
    ubb_map: &[Vec<Option<Chip>>],
) -> Result<(), Error> {
    for edge in edges_to_disable {
        let mask = convert_edges_to_disable_to_port_disable_mask(&edge)?;
        flash_asic(
            ubb_map[edge.node.0][edge.node.1].as_ref().unwrap(),
            ETH_PARAM_PORT_DISABLE,
            &mask,
        )?;
    }
    Ok(())
}

/// Isolate a single UBB tray by disabling the appropriate external Ethernet links.
/// Isolating UBB means disabling the following connections on each asic:
///
/// asic 1: disable 3_0, 7_4            asic 5: disable 7_4, 15_12
/// asic 2: disable 3_0                 asic 6: disable 15_12
/// asic 3: disable 3_0                 asic 7: disable 15_12
/// asic 4: disable 3_0, 11_8           asic 8: disable 15_12, 11_8
pub fn isolate_ubb(ubb_num: usize, ubb_map: &[Vec<Option<Chip>>]) -> Result<(), Error> {
    println!("{}", format!("Isolating UBB {}...", ubb_num).yellow());

    let ubb = &ubb_map[ubb_num - 1];
    flash_asic(
        ubb[0].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(true, true, false, false),
    )?;

    flash_asic(
        ubb[1].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(true, false, false, false),
    )?;

    flash_asic(
        ubb[2].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(true, false, false, false),
    )?;

    flash_asic(
        ubb[3].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(true, false, true, false),
    )?;

    flash_asic(
        ubb[4].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(false, true, false, true),
    )?;

    flash_asic(
        ubb[5].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(false, false, false, true),
    )?;

    flash_asic(
        ubb[6].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(false, false, false, true),
    )?;

    flash_asic(
        ubb[7].as_ref().unwrap(),
        ETH_PARAM_PORT_DISABLE,
        &generate_port_disable_mask(false, false, true, true),
    )?;

    println!("{}", format!("Successfully isolated UBB {}.", ubb_num).green());

    Ok(())
}

/// Restore all ASICs in a UBB tray to default Ethernet settings (no ports disabled).
/// This is done by flashing all ASICs with a 0 port disable mask.
pub fn set_default_ubb_eth_disables(
    ubb_num: usize,
    ubb_device_map: &[Vec<Option<Chip>>],
) -> Result<(), Error> {
    println!("{}", format!("Setting default eth disables for UBB {}...", ubb_num).yellow());
    let ubb = &ubb_device_map[ubb_num - 1];
    for asic in ubb.iter().flatten() {
        flash_asic(asic, ETH_PARAM_PORT_DISABLE, &[0, 0, 0, 0])?;
    }
    Ok(())
}

/// Reset the ASICs in a UBB tray to default Ethernet settings.
/// This is done by flashing all ASICs with a 0 port disable mask.
pub fn reset_group_of_asics(
    group_of_asics: Vec<GraphCoords>,
    ubb_device_map: &[Vec<Option<Chip>>],
) -> Result<(), Error> {
    for coord in group_of_asics.iter() {
        let asic = ubb_device_map[coord.0][coord.1].as_ref().unwrap();
        flash_asic(asic, ETH_PARAM_PORT_DISABLE, &[0, 0, 0, 0])?;
        println!("{}", format!("Restored ASIC at UBB:{}, ASIC:{}", coord.0 + 1, coord.1 + 1).blue());
    }
    Ok(())
}
