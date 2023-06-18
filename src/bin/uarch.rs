use vondel::uarch::{
    mem::{CtrlStore, Ram},
    Computer,
};

pub fn main() {
    // trying to do 5 * 6 (dunno why)
    let mut mem = Ram::new();
    mem.load(0, [5, 6]);

    // |   NEXT  |JAM|   ALU  |          C BUS         |MEM|  A  |  B  |
    // |---------|---|--------|------------------------|---|-----|-----|
    // |000000000|000|00000000|000000000000000000000000|000|00000|00000|
    //
    // R15: Stores the sum
    // R14: Stores the 5
    // R13: Stores the 6
    #[allow(clippy::unusual_byte_groupings)]
    let mcode = [
        // READ word from memory (number 5 because MAR = 0 by default)
        0b000000001_000_00000000_00000000000000000000_010_11111_11111,
        // LOAD the value readed into R15 and R14
        0b000000010_000_00011000_00000000000000000011_000_00000_11111,
        // MAR = 1 (next word address)
        0b000000011_000_00110001_01000000000000000000_000_11111_11111,
        // READ word FROM memory (number 6)
        0b000000100_000_00000000_00000000000000000000_010_11111_11111,
        // LOAD the value readed into R13 and R12 (just to display later)
        0b000000101_000_00011000_00000000000000001100_000_00000_11111,
        // R13 <- R13 - 1 and JUMP if 0
        0b000000110_001_00110110_00000000000000000100_000_11111_10000,
        // R15 <- R15 + R14
        0b000000101_000_00111100_00000000000000000001_000_10111_10001,
    ];
    let firmware = CtrlStore::builder()
        .load(0, mcode)
        // end when JUMP
        .set(0b100000110, CtrlStore::TERM)
        .build();

    let mut comp = Computer::new(mem, firmware);
    comp.exec();

    let regs = &comp.regs().gen;
    println!(
        "{} x {} = {}",
        regs.get(14).unwrap(),
        regs.get(12).unwrap(),
        regs.get(15).unwrap()
    );
    println!("Cycles: {}", comp.cycles());
}
