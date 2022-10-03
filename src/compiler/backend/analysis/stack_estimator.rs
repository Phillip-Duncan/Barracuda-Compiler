use std::cmp::max;
use crate::compiler::program_code::instructions::BarracudaInstructions;
use crate::compiler::program_code::ProgramCode;

pub struct StackEstimator {

    /// Note max depth is used as a precaution since stack estimator works of program code
    /// it is possible to define a endless recursive program using this representation. Its
    /// unlikely that the backend would generate a recursive program to this degree.
    max_depth: usize,
    max_depth_reached: bool
}

impl StackEstimator {

    /// Follows an execution path in program code and returns the max stack size estimate from following
    /// that path to either the end of the program or the return of a function. This is identified as a
    /// non constant addressed GOTO as this is presently the only context for a return.
    /// @code: ProgramCode to follow the execution of
    /// @pc: Program Counter to start following from
    /// @stack_size: Stack size estimate entering this execution path
    /// @depth: Current recursive depth of following these statements. On reaching self.max_depth
    ///         stack size is returned and the flag self.max_depth_reached is set.
    /// @return: max_expected_stack_size from following the execution path
    fn follow_execution_path(&mut self, code: &ProgramCode, pc: usize, stack_size: usize, depth: usize) -> usize {
        let mut pc = pc;
        let mut stack_size = stack_size;
        let mut max_stack_size = 0;

        // Used to know the goto address of static jumps
        let mut last_value = None;

        // Check recursive depth
        if depth >= self.max_depth {
            self.max_depth_reached = true;
            return stack_size;
        }

        // Follow Execution
        while let Some(instruction) = code.instructions.get(pc) {
            max_stack_size = max(max_stack_size, stack_size);

            match instruction {
                BarracudaInstructions::OP => {
                    if let Some(op) = code.operations.get(pc) {
                        stack_size -= op.consume() as usize;
                        stack_size += op.produce() as usize;
                    }

                    pc += 1;
                }
                BarracudaInstructions::VALUE => {
                    stack_size += 1;
                    last_value = code.values.get(pc);
                    pc += 1;
                }
                BarracudaInstructions::GOTO => {
                    // Address
                    stack_size -= 1;

                    // If no last value is set then the function must be returning from a call
                    // as this is the only context for this action at present
                    if let Some(address) = last_value {
                        pc = *address as usize;
                    } else {
                        break;
                    }
                }
                BarracudaInstructions::GOTO_IF => {
                    // Address + Condition
                    stack_size -= 2;

                    if let Some(address) = last_value {
                        let false_pc = *address as usize;

                        // Follow true path
                        max_stack_size = max(max_stack_size,self.follow_execution_path(code, pc + 1, stack_size, depth + 1));

                        // Follow false path
                        pc = false_pc;
                    } else {
                        break;
                    }
                }

                BarracudaInstructions::LOOP_ENTRY |
                BarracudaInstructions::LOOP_END => {
                    unimplemented!()    // TODO(Connor): Saving implementation for when backend uses loop_entry/exit
                }
            }

            // Remove *last* value if no longer the immediate previous value
            if BarracudaInstructions::VALUE != *instruction  {
                last_value = None;
            }
        };

        return max_stack_size;
    }

    /// Estimates the max stack size from executing of a program.
    /// If max_depth_reached is true then the estimate is not complete.
    /// @code: ProgramCode to follow the execution of
    /// @max_depth: Max recursive depth to follow when doing branch analysis
    /// @return (max_stacksize: usize, max_depth_reached: bool)
    pub fn estimate_max_stacksize(code: &ProgramCode, max_depth: usize) -> (usize, bool) {
        let mut estimator = Self {
            max_depth,
            max_depth_reached: false
        };

        let max_stacksize = estimator.follow_execution_path(code, 0, 0, 0);

        return (max_stacksize, estimator.max_depth_reached);
    }
}
