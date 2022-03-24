use crate::emulator::{self};
use crate::emulator::ops::MathStackOperators::*;
use crate::emulator::instructions::MathStackInstructions::*;
use std::io;

/*
    These examples are from the VM documentation of example programs. It does not
    show full behavioural emulation but at least indicates that it can at least
    run these programs. These examples can be found:
    https://docs.google.com/document/d/1eY3zTsh10cGsLRoAzGmqvKJDEF9k6ves3u1UW2keCSw/edit
 */

fn assert_stack_equal(context: emulator::ThreadContext, other: Vec<f64>, delta: f64) {
    let stack = context.get_stack();

    assert_eq!(stack.len(), other.len());

    for i in 0..stack.len() {
        assert_relative_eq!(stack[i], other[i], max_relative=delta);
    }
}

#[test]
fn vm_example_1() {
    let mut context = emulator::ThreadContext::new(5,
                                         vec![5.0, 6.0, 10.0],
                                         vec![DIV, MUL, LDB, MUL, LDB, SIN, ADD],
                                         vec![OP, OP, OP, OP, OP, OP, OP, VALUE, VALUE, VALUE],
                                         Box::new(io::stdout()));
    context.set_env_var(1, 1.5).unwrap();
    context.run_till_halt().unwrap();

    assert_relative_eq!(context.get_stack()[0], -4.444, max_relative=0.001);
}

#[test]
fn vm_example_2() {
    let mut context = emulator::ThreadContext::new(10,
                                                   vec![1.0, 10.0, 0.0, 1.0, 10.0, 0.0, 10.0, 0.0, 5.0],
                                                   vec![ADD, ADD],
                                                   vec![LOOP_END, LOOP_END, OP, VALUE, LOOP_ENTRY, VALUE, VALUE, LOOP_END, OP, VALUE, LOOP_ENTRY, VALUE, VALUE, LOOP_ENTRY, VALUE, VALUE, VALUE],
                                                   Box::new(io::stdout()));
    context.run_till_halt().unwrap();

    assert_relative_eq!(context.get_stack()[0], 205.0, max_relative=0.001);
}

#[test]
fn vm_example_3() {
    let mut context = emulator::ThreadContext::new(6,
                                                   vec![12.0, 5.0, 6.0, 10.0],
                                                   vec![DIV,MUL,LDB,MUL,LDB,SIN,ADD],
                                                   vec![OP, GOTO, VALUE, OP, OP, OP, OP, OP, OP, VALUE, VALUE, VALUE],
                                                   Box::new(io::stdout()));
    context.set_env_var(1, 1.5).unwrap();
    context.run_till_halt().unwrap();

    assert_stack_equal(context, vec![10.0, f64::sin(6.0+5.0)*1.5*1.5], 0.001)
}

#[test]
fn vm_example_4() {
    let mut context = emulator::ThreadContext::new(6,
                                                   vec![10.0, 1.0, 3.0, 0.0, 7.0, 5.0],
                                                   vec![ADD],
                                                   vec![/* 20?, */ GOTO_IF, VALUE, VALUE, GOTO, VALUE, OP, VALUE, GOTO, VALUE, VALUE],
                                                   Box::new(io::stdout()));
    context.run_till_halt().unwrap();

    // TODO(Connor): Find out what the expected output here should be
    assert!(false);
}

#[test]
fn vm_example_rule_110() {
    let board_size: f64 = 50.0;
    let bs = board_size;

    let output_buffer = io::Cursor::new(Vec::new());
    let mut context = emulator::ThreadContext::new(66,
                                                   vec![1.0,110.0,4.0,7.0,1.0,4.0,(bs-1.0),1.0,4.0,4.0,1.0,10.0,32.0,42.0,4.0,(bs),0.0,(bs-2.0),0.0,1.0,(bs-2.0)*4.0,(bs)*4.0],
                                                   vec![DROP,DROP,SWAP,WRITE,AND,RSHIFT,SWAP,OVER,SUB_PTR,OVER,OR,READ,OVER,AND,LSHIFT,SWAP,ADD_PTR,ADD_PTR,OVER,OR,READ,ADD_PTR,OVER,LSHIFT,READ,DUP,PRINTC,DROP,PRINTC,TERNARY,READ,DUP, ADD_PTR,DUP,WRITE,ADD_PTR,DUP,MALLOC],
                                                   vec![/* Unknown? */], // TODO(Connor): Find out what the expected instructions should be
                                                   Box::new(output_buffer));
    // TODO(Connor): init stack with default values
    // {100,0,0,100,0,0,0,1,0,0,1,0,0,1,0,0,0,0,0,1,0,1,0,0,1,99,
    //  1,1,0,1,0,0,0,0,1,0,0,1,0,0,0,1,0,100,
    //  0,0,1,1,0,0,0,1,99,1,1,0,99,1,1,0,1,0,1,0,0,1}
    context.run_till_halt().unwrap();

    // TODO(Connor): Find out what the expected output here should be
    assert!(false);
}