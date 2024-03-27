contract;

/// This does not work:
const CONSTANT_A: u64 = 0;
const CONSTANT_B: u64 = CONSTANT_A + 1;
const CONSTANT_C: u64 = CONSTANT_B + 1;

/// This works:
// const CONSTANT_A: u64 = 0;
// const CONSTANT_B: u64 = 1;
// const CONSTANT_C: u64 = 2;

abi MyContract {
    fn match_with_constants(value: u64) -> u64;
}

impl MyContract for Contract {
    fn match_with_constants(value: u64) -> u64 {
        

        let ret = match value {
            CONSTANT_A => CONSTANT_A,
            CONSTANT_B => CONSTANT_B,
            CONSTANT_C => CONSTANT_C,
            _ => revert(0),
        };

        ret
    }
}