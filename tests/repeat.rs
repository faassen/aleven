use aleven::parse_program;
use aleven::run::{compiled, interpreted, Run};
use parameterized::parameterized;

#[parameterized(run={compiled, interpreted})]
fn test_repeat(run: Run) {
    let program = parse_program(
        "
    repeat main 10 {
        r1 = addi r4 30
        sb r2 0 = r1
        r2 = addi r2 1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];

    run(&program, &mut memory);

    for value in memory.iter().take(10) {
        assert_eq!(*value, 30);
    }
    assert_eq!(memory[10], 0);
}
