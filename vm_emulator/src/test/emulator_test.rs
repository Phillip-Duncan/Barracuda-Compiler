use crate::emulator::{self};
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
use std::io::{Cursor, Read};
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
                                                   vec![0.0,0.0,0.0,1.0,0.0,10.0,0.0,0.0,0.0,1.0,0.0,10.0,0.0,0.0,10.0,0.0,5.0],
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
                                                   vec![0.0, 0.0, 3.0, 0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 7.0, 5.0],
                                                   vec![FIXED(NULL), FIXED(ADD)],
                                                               // 10    9        8      7      6    5      4    3     2     1       0
                                                   vec![OP, GOTO_IF, VALUE, VALUE, GOTO, VALUE, OP, VALUE, GOTO, VALUE, VALUE],
                                                   Rc::new(RefCell::new(io::stdout())));
    context.run_till_halt().unwrap();
    assert_stack_equal(context, vec![6.0], 0.001);
}

#[test]
fn vm_example_rule_110() {
    let board_size: f64 = 50.0;
    let bs = board_size;

    let expected_output = "                                               *  
                                              **  
                                             ***  
                                            ** *  
                                           *****  
                                          **   *  
                                         ***  **  
                                        ** * ***  
                                       ******* *  
                                      **     ***  
                                     ***    ** *  
                                    ** *   *****  
                                   *****  **   *  
                                  **   * ***  **  
                                 ***  **** * ***  
                                ** * **  ***** *  
                               ******** **   ***  
                              **      ****  ** *  
                             ***     **  * *****  
                            ** *    *** ****   *  
                           *****   ** ***  *  **  
                          **   *  ***** * ** ***  
                         ***  ** **   ******** *  
                        ** * ******  **      ***  
                       *******    * ***     ** *  
                      **     *   **** *    *****  
                     ***    **  **  ***   **   *  
                    ** *   *** *** ** *  ***  **  
                   *****  ** *** ****** ** * ***  
                  **   * ***** ***    ******** *  
                 ***  ****   *** *   **      ***  
                ** * **  *  ** ***  ***     ** *  
               ******** ** ***** * ** *    *****  
              **      ******   ********   **   *  
             ***     **    *  **      *  ***  **  
            ** *    ***   ** ***     ** ** * ***  
           *****   ** *  ***** *    ********** *  
          **   *  ***** **   ***   **        ***  
         ***  ** **   ****  ** *  ***       ** *  
        ** * ******  **  * ***** ** *      *****  
       *******    * *** ****   ******     **   *  
      **     *   **** ***  *  **    *    ***  **  
     ***    **  **  *** * ** ***   **   ** * ***  
    ** *   *** *** ** ******** *  ***  ******* *  
   *****  ** *** ******      *** ** * **     ***  
  **   * ***** ***    *     ** *********    ** *  
 ***  ****   *** *   **    *****       *   *****  
** * **  *  ** ***  ***   **   *      **  **   *  
";

    let program_output: Rc<RefCell<Cursor<Vec<u8>>>> = Rc::new(RefCell::new(Cursor::new(Vec::new())));
    let mut context = emulator::ThreadContext::new(200,
                                                   vec![0.0,0.0,0.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,110.0,0.0,0.0,4.0,0.0,0.0,0.0,0.0,0.0,7.0,0.0,1.0,0.0,0.0,4.0,0.0,(bs-1.0),1.0,0.0,4.0,0.0,0.0,0.0,0.0,4.0,0.0,0.0,1.0,0.0,0.0,0.0,10.0,0.0,0.0,0.0,0.0,32.0,42.0,0.0,0.0,0.0,4.0,0.0,(bs),0.0,0.0,0.0,(bs-2.0),0.0,0.0,1.0,0.0,(bs-2.0)*4.0,0.0,0.0,4.0,0.0,(bs+2.0)*4.0],
                                                   vec![FIXED(NULL),FIXED(DROP),FIXED(DROP),FIXED(NULL),FIXED(SWAP),FIXED(WRITE),FIXED(AND),FIXED(NULL),FIXED(RSHIFT),FIXED(SWAP),FIXED(NULL),FIXED(OVER),FIXED(SUB_PTR),FIXED(NULL),FIXED(OVER),FIXED(OR),FIXED(READ), FIXED(OVER),FIXED(AND),FIXED(NULL),FIXED(LSHIFT),FIXED(NULL),FIXED(SWAP),FIXED(ADD_PTR), FIXED(NULL),FIXED(NULL),FIXED(NULL),FIXED(NULL),FIXED(ADD_PTR),FIXED(NULL),FIXED(OVER),FIXED(OR),FIXED(READ),FIXED(ADD_PTR),FIXED(NULL),FIXED(OVER),FIXED(LSHIFT),FIXED(NULL),FIXED(READ),FIXED(DUP),FIXED(PRINTC),FIXED(NULL),FIXED(DROP),FIXED(NULL),FIXED(PRINTC),FIXED(TERNARY),FIXED(NULL),FIXED(NULL),FIXED(READ),FIXED(DUP), FIXED(ADD_PTR),FIXED(NULL),FIXED(NULL),FIXED(NULL),FIXED(NULL),FIXED(DUP),FIXED(NULL),FIXED(NULL),FIXED(NULL),FIXED(WRITE),FIXED(NULL),FIXED(ADD_PTR),FIXED(NULL),FIXED(DUP),FIXED(ADD_PTR), FIXED(NULL), FIXED(MALLOC),FIXED(NULL)],
                                                   vec![LOOP_END,OP,OP,LOOP_END,OP,OP,OP,VALUE,OP,OP,VALUE,OP,OP,VALUE,OP,OP,OP,OP,OP,VALUE,OP,VALUE,OP,OP,VALUE,LOOP_ENTRY, VALUE,VALUE,OP,VALUE,OP,OP,OP,OP,VALUE,OP,OP,VALUE,OP,OP,OP,VALUE,OP,LOOP_END,OP,OP,VALUE,VALUE,OP,OP,OP,VALUE,LOOP_ENTRY,VALUE,VALUE,OP,LOOP_ENTRY,VALUE,VALUE,OP,VALUE,OP,VALUE,OP,OP, VALUE, OP,VALUE],
                                                   program_output.clone());
    context.run_till_halt().unwrap();
    let mut buffer = String::new();
    let mut out = program_output.borrow_mut();
    out.set_position(0);
    out.read_to_string(&mut buffer).unwrap();

    assert_eq!(expected_output, buffer.as_str());
}