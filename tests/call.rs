use aleven::parse_program;
use aleven::run::{compiled, interpreted, Run};
use parameterized::parameterized;

#[parameterized(run={compiled, interpreted})]
fn test_call(run: Run) {
    let program = parse_program(
        "
    func main {
      r2 = lb r1 0
      call sub
    }

    func sub {
      sb r3 10 = r2
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 11;

    run(&program, &mut memory);

    assert_eq!(memory[10], 11);
}

#[parameterized(run={compiled, interpreted})]
fn test_nested_call(run: Run) {
    let program = parse_program(
        "
    func main {
      r1 = lb r31 0
      r2 = lb r31 1
      r3 = lb r31 2
      r4 = lb r31 3
      call sub
      sb r31 13 = r4
    }

    func sub {
      sb r31 10 = r1
      call sub_sub
      sb r31 12 = r3
    }

    func sub_sub {
      sb r31 11 = r2
    }
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 11;
    memory[1] = 12;
    memory[2] = 13;
    memory[3] = 14;

    run(&program, &mut memory);

    assert_eq!(memory[10], 11);
    assert_eq!(memory[11], 12);
    assert_eq!(memory[12], 13);
    assert_eq!(memory[13], 14);
}

#[parameterized(run={compiled, interpreted})]
fn test_no_recursion_basic(run: Run) {
    let program = parse_program(
        "
       func main { 
          call main 
       }",
    )
    .unwrap();
    let mut memory = [0u8; 64];
    run(&program, &mut memory);
}
