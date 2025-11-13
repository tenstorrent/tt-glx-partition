// This file houses all the functions to access and flash the chips
use color_eyre::eyre::{Error, Result};
use luwen::api::{ChipImpl, chip::Chip};

// Magic constants for UBB tray to PCIe bus mapping
pub const fn wh_pcie_to_ubb_tray(c: char) -> Option<usize> {
    match c {
        'c' => Some(0),
        '8' => Some(1),
        '0' => Some(2),
        '4' => Some(3),
        _ => None,
    }
}

pub const fn bh_pcie_to_ubb_tray(c: char) -> Option<usize> {
    match c {
        '0' => Some(0),
        '4' => Some(1),
        'c' => Some(2),
        '8' => Some(3),
        _ => None,
    }
}

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
// Param table addresses in the SPI relevant to tt-topology
const ETH_PARAM_BASE_ADDR: u32 = 0x21100;
const ETH_PARAM_PORT_DISABLE: u32 = ETH_PARAM_BASE_ADDR + (0x4) * 2;

// ANSI color codes for terminal output
pub struct CmdLineColor;
impl CmdLineColor {
    pub const PURPLE: &'static str = "\x1b[35m";
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const ENDC: &'static str = "\x1b[0m";
}

pub fn get_ubb_device_map(devices: Vec<Chip>) -> Result<Vec<Vec<Option<Chip>>>, Error> {
    /*
    Use the pcie bus ID to map all the asics to their respective UBBs.
    And then return these objects in a nested vector where the outer vector is UBB num - 1 and inner vector is ASIC num - 1.

    Return type: [[UBB 1 Chips][UBB 2 Chips][UBB 3 Chips][UBB 4 Chips]]
    */

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
                println!(
                    "{}Mapping BH Device {} (Bus: 0x{}) to UBB Slot {}, ASIC Slot {}{}",
                    CmdLineColor::GREEN,
                    dev_num,
                    bus_hex,
                    ubb_slot,
                    asic_slot_num,
                    CmdLineColor::ENDC
                );
                ubb_map[ubb_slot][asic_slot_num] = Some(dev);
            } else {
                println!(
                    "{}Device {} is neither WH nor BH chip, skipping...{}",
                    CmdLineColor::RED,
                    dev_num,
                    CmdLineColor::ENDC
                );
            }
        }
    }

    Ok(ubb_map)
}

pub fn flash_asic(dev: &Chip, addr: u32, data: &[u8]) -> Result<(), Error> {
    // flash the WH ASIC, will add BH support later
    println!(
        "{}Flashing ASIC at address {:#x} with data: {}{}",
        CmdLineColor::PURPLE,
        addr,
        hex::encode(data),
        CmdLineColor::ENDC
    );
    if let Some(wh) = dev.as_wh() {
        wh.spi_write(addr, data).unwrap();
        println!(
            "{}Successfully flashed ASIC at address {:#x}{}",
            CmdLineColor::GREEN,
            addr,
            CmdLineColor::ENDC
        );
        Ok(())
    } else if let Some(bh) = dev.as_bh() {
        bh.spi_write(addr, data).unwrap();
        println!(
            "{}Successfully flashed ASIC at address {:#x}{}",
            CmdLineColor::GREEN,
            addr,
            CmdLineColor::ENDC
        );
        Ok(())
    } else {
        let error_msg = format!(
            "Device is not a WH or BH chip, cannot flash ASIC at address {:#x}",
            addr
        );
        println!(
            "{}Error: {}{}",
            CmdLineColor::RED,
            error_msg,
            CmdLineColor::ENDC
        );
        Err(Error::msg(error_msg))
    }
}

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

pub fn isolate_ubb(ubb_num: usize, ubb_map: &[Vec<Option<Chip>>]) -> Result<(), Error> {
    /*
        Given a UBB, flash all asics to disable external eth connections.
        UBB Asic port layout:

            +-------[7:4]-------+
            |                   |
            |                   |
        [3:0]        ASIC       [15:12]
            |                   |
            |                   |
            +------[11:8]-------+

        In each UBB layout, the ASICs are arranged as follows:

        ASIC numbers in each UBB:
        +-------+
        | 1 | 5 |
        +-------+
        | 2 | 6 |
        +-------+
        | 3 | 7 |
        +-------+
        | 4 | 8 |
        +-------+

        Isolating UBB means disabling the following connections on each asic.

        asic 1: disable 3_0, 7_4            asic 5: disable 7_4, 15_12
        asic 2: disable 3_0                 asic 6: disable 15_12
        asic 3: disable 3_0                 asic 7: disable 15_12
        asic 4: disable 3_0, 11_8           asic 8: disable 15_12, 11_8
    */
    println!(
        "{}Isolating UBB {}...{}",
        CmdLineColor::YELLOW,
        ubb_num,
        CmdLineColor::ENDC
    );

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

    println!(
        "{}Successfully isolated UBB {}.{}",
        CmdLineColor::GREEN,
        ubb_num,
        CmdLineColor::ENDC
    );

    Ok(())
}

pub fn set_default_ubb_eth_disables(
    ubb_num: usize,
    ubb_device_map: &[Vec<Option<Chip>>],
) -> Result<(), Error> {
    // reset all UBB eth disables to 0
    println!(
        "{}Setting default eth disables for UBB {}...{}",
        CmdLineColor::YELLOW,
        ubb_num,
        CmdLineColor::ENDC
    );
    let ubb = &ubb_device_map[ubb_num - 1];
    for asic in ubb.iter().flatten() {
        flash_asic(asic, ETH_PARAM_PORT_DISABLE, &[0, 0, 0, 0])?;
    }
    Ok(())
}
