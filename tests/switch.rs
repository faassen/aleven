// use aleven::parse_program;
// use aleven::run::{compiled, interpreted, Run};
// use parameterized::parameterized;

// #[parameterized(run={compiled, interpreted})]
// fn test_switch(run: Run) {
//     let program = parse_program(
//         "
//     func main {
//       r1 = lb r31 0
//       r2 = lb r31 1
//       r3 = lb r31 2
//       r4 = lb r31 3
//       switch r1 bar1 3
//     }

//     func bar1 {
//       sb r31 10 = r2
//     }

//     func bar2 {
//       sb r31 10 = r3
//     }

//     func bar3 {
//         sb r31 10 = r4
//     }
//     ",
//     )
//     .unwrap();

//     let mut memory = [0u8; 64];
//     memory[0] = 0;
//     memory[1] = 1;
//     memory[2] = 2;
//     memory[3] = 3;
//     let stored_memory = memory;

//     run(&program, &mut memory);

//     assert_eq!(memory[10], 1);

//     memory = stored_memory;
//     memory[0] = 1;
//     run(&program, &mut memory);
//     assert_eq!(memory[10], 2);

//     memory = stored_memory;
//     memory[0] = 2;
//     run(&program, &mut memory);
//     assert_eq!(memory[10], 3);
// }

// #[parameterized(run={compiled, interpreted})]
// fn test_switch_more_than_amount_wraps(run: Run) {
//     let program = parse_program(
//         "
//     func main {
//       r1 = lb r31 0
//       r2 = lb r31 1
//       r3 = lb r31 2
//       r4 = lb r31 3
//       switch r1 bar1 3
//     }

//     func bar1 {
//       sb r31 10 = r2
//     }

//     func bar2 {
//       sb r31 10 = r3
//     }

//     func bar3 {
//         sb r31 10 = r4
//     }
//     ",
//     )
//     .unwrap();

//     let mut memory = [0u8; 64];
//     memory[0] = 3;
//     memory[1] = 1;
//     memory[2] = 2;
//     memory[3] = 3;
//     let stored_memory = memory;

//     run(&program, &mut memory);

//     assert_eq!(memory[10], 1);

//     memory = stored_memory;
//     memory[0] = 7;
//     run(&program, &mut memory);
//     assert_eq!(memory[10], 2);

//     memory = stored_memory;
//     memory[0] = 14;
//     run(&program, &mut memory);
//     assert_eq!(memory[10], 3);
// }
