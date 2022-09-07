use aleven::parse_program;
use aleven::run::{compiled, interpreted, Run};
use parameterized::parameterized;

#[parameterized(run={compiled, interpreted})]
fn test_beq_simple(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        beq r2 r3 end
        r4 = lb r1 2
        sb r5 10 = r4
        target end
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_beq_earlier_target_means_nop(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        target earlier
        beq r2 r3 earlier # exists but before me
        r4 = lb r1 2
        sb r5 10 = r4
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // since noop, branch happened
    assert_eq!(memory[10], 30);

    // in the other case, it's the same noop, so store happens
    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;
    run(&program, &mut memory);
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_addi_after_beq(run: Run) {
    let program = parse_program(
        "
    func main {
        r24 = lh r24 8728
        beq r31 r31 foo
        r24 = addi r24 6168
        target foo
        r24 = addi r24 0
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    run(&program, &mut memory);
}

#[parameterized(run={compiled, interpreted})]
fn test_bne_simple(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bne r2 r3 f1
        r4 = lb r1 2
        sb r5 10 = r4
        target f1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 15;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_blt_simple(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        blt r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 15;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_blt_negative(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        blt r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = -1_i8 as u8;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bltu_simple(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bltu r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 5;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    // bltu is unsigned, so -1 is actually greater than 10
    let mut memory = [0u8; 64];
    memory[0] = -1_i8 as u8;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bge_simple(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bge r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bge_equal(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bge r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bge_negative(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bge r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = -1_i8 as u8;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = -1_i8 as u8;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bgeu_simple(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bgeu r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = 10;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bgeu_equal(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bgeu r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(run={compiled, interpreted})]
fn test_bgeu_negative(run: Run) {
    let program = parse_program(
        "
    func main {
        r2 = lb r1 0
        r3 = lb r1 1
        bgeu r2 r3 t1
        r4 = lb r1 2
        sb r5 10 = r4
        target t1
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = -1_i8 as u8;
    memory[1] = 20;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = -1_i8 as u8;
    memory[2] = 30;

    run(&program, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}
