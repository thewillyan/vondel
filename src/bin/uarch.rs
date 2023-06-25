use std::{
    fs::File,
    io::{BufReader, Read},
};

use anyhow::Result;
use clap::Parser;
use vondel::uarch::{
    cli::UArchCli,
    mem::{CtrlStore, Ram},
    Computer,
};

fn read_ram(file: &str) -> Result<Ram> {
    let file = File::open(file)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut result = Vec::new();
    for i in (0..buffer.len()).step_by(4) {
        let mut word = [0u8; 4];
        word.copy_from_slice(&buffer[i..i + 4]);
        result.push(u32::from_le_bytes(word));
    }
    let mut ram = Ram::new();
    ram.load(0, result);

    Ok(ram)
}

fn read_firmware(file: &str) -> Result<CtrlStore> {
    let file = File::open(file)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut result = Vec::new();
    for i in (0..buffer.len()).step_by(8) {
        let mut word = [0u8; 8];
        word.copy_from_slice(&buffer[i..i + 8]);
        result.push(u64::from_le_bytes(word));
    }
    let firmware = CtrlStore::builder().load(0, result).build();

    Ok(firmware)
}

pub fn main() -> Result<()> {
    let cli = UArchCli::parse();
    let ram = read_ram(&cli.ram)?;
    let firmware = read_firmware(&cli.rom)?;

    for (idx, value) in firmware.firmware().iter().enumerate() {
        if *value != 0 {
            println!("Address: {}", idx);
            println!("{:064b}", value);
        }
    }

    let mut comp = Computer::new(ram, firmware);
    comp.exec();
    let regs = &comp.regs().gen;

    println!("Value of register 'ra': {}", regs.get(0).unwrap());
    for i in 0..15 {
        println!("Value of register r{}: {}", i, regs.get(i).unwrap());
    }

    if cli.cycles {
        println!("Cycles: {}", comp.cycles());
    }

    Ok(())
}
