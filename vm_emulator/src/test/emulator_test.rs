use crate::emulator;
use barracuda_common::{
    BarracudaInstructions::*,
    FixedBarracudaOperators::*,
    VariableBarracudaOperators::*,
    BarracudaOperators::*
};

use std::io;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::EnvironmentVariable;

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
        assert_relative_eq!(stack[i].into_f64(), other[i], max_relative=delta);
    }
}

#[test]
fn vm_example_1() {
    let mut context = emulator::ThreadContext::new(5,
                                         vec![5.0, 6.0, 10.0],
                                         vec![FIXED(DIV), FIXED(MUL), VARIABLE(LDNX(1)), FIXED(MUL), VARIABLE(LDNX(1)), FIXED(SIN), FIXED(ADD)],
                                         vec![OP, OP, OP, OP, OP, OP, OP, VALUE, VALUE, VALUE],
                                                   Rc::new(RefCell::new(io::stdout())));
    context = context.with_env_vars(HashMap::from([
        (1, EnvironmentVariable::new(String::from("b"), 1, 1.5))
    ]));

    context.run_till_halt().unwrap();

    assert_relative_eq!(context.get_stack()[0].into_f64(), -4.444, max_relative=0.001);
}

#[test]
fn vm_example_2() {
    let mut context = emulator::ThreadContext::new(10,
                                                   vec![0.0,0.0,0.0,1.0,0.0,f64::from_be_bytes((10 as u64).to_be_bytes()),0.0,0.0,0.0,1.0,0.0,f64::from_be_bytes((10 as u64).to_be_bytes()),0.0,0.0,f64::from_be_bytes((10 as u64).to_be_bytes()),0.0,5.0],
                                                   vec![FIXED(ADD), FIXED(ADD)],
                                                   vec![LOOP_END, LOOP_END, OP, VALUE, LOOP_ENTRY, VALUE, VALUE, LOOP_END, OP, VALUE, LOOP_ENTRY, VALUE, VALUE, LOOP_ENTRY, VALUE, VALUE, VALUE],
                                                   Rc::new(RefCell::new(io::stdout())));
    context.run_till_halt().unwrap();

    assert_relative_eq!(context.get_stack()[0].into_f64(), 205.0, max_relative=0.001);
}

#[test]
fn vm_example_3() {
    let mut context = emulator::ThreadContext::new(6,
                                                   vec![f64::from_be_bytes((12 as u64).to_be_bytes()), 5.0, 6.0, 10.0],
                                                   vec![FIXED(DIV),FIXED(MUL),VARIABLE(LDNX(1)),FIXED(MUL),VARIABLE(LDNX(1)),FIXED(SIN),FIXED(ADD)],
                                                   vec![OP, GOTO, VALUE, OP, OP, OP, OP, OP, OP, VALUE, VALUE, VALUE],
                                                   Rc::new(RefCell::new(io::stdout())));
    context = context.with_env_vars(HashMap::from([
        (1, EnvironmentVariable::new(String::from("b"), 1, 1.5))
    ]));
    context.run_till_halt().unwrap();

    assert_stack_equal(context, vec![10.0, f64::sin(6.0+5.0)*1.5*1.5], 0.001)
}

#[test]
fn vm_example_4() {
    let mut context = emulator::ThreadContext::new(6,
                                                   vec![0.0, 0.0, f64::from_be_bytes((3 as u64).to_be_bytes()), 0.0, 0.0, f64::from_be_bytes((10 as u64).to_be_bytes()), 0.0, 1.0, 0.0, f64::from_be_bytes((7 as u64).to_be_bytes()), 5.0],
                                                   vec![FIXED(NULL), FIXED(ADD)],
                                                               // 10    9        8      7      6    5      4    3     2     1       0
                                                   vec![OP, GOTO_IF, VALUE, VALUE, GOTO, VALUE, OP, VALUE, GOTO, VALUE, VALUE],
                                                   Rc::new(RefCell::new(io::stdout())));
    context.run_till_halt().unwrap();
    assert_stack_equal(context, vec![6.0], 0.001);
}