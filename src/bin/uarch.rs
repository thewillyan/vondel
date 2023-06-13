use vondel::uarch::{
    mem::{CtrlStore, Ram, Register},
    Computer,
};

pub fn main() {
    // trying to do 5 * 6 (dunno why)
    // both are u16 for now
    let data = [(5 << 24) | 6];
    let mut mem = Ram::default();
    mem.load(0, data);

    // |   NEXT  |JAM|   ALU  | C BUS  |MEM| A  | B |
    // |---------|---|--------|--------|---|----|---|
    // |000000000|000|00000000|00000000|000|0000|000|
    #[allow(clippy::unusual_byte_groupings)]
    let mcode = [
        // SOR <- 5
        0b000000001_000_00011000_00000001_001_0101_111,
        // OB <- 6
        0b000000010_000_00011000_00000010_001_0101_111,
        // OA <- OA + SOR
        0b000000011_000_00111100_00000100_000_1001_101,
        // OB <- OB - 1 and JUMP if 0
        0b000000010_001_00110110_00000010_000_1111_100,
    ];
    let firmware = CtrlStore::builder()
        .load(0, mcode)
        // end when JUMP
        .set(0b100000010, CtrlStore::TERM)
        .build();

    let mut comp = Computer::new(mem, firmware);
    comp.exec();

    println!("5 x 6 = OA = {}", comp.regs().gen.oa.get());
    println!("OB: {}", comp.regs().gen.ob.get());
    println!("SOR: {}", comp.regs().gen.sor.get());
    println!("Cycles: {}", comp.cycles());
}
