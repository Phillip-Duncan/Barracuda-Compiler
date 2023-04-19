use barracuda_common::BarracudaInstructions;
use barracuda_common::{
    BarracudaOperators,
    FixedBarracudaOperators::*,
    VariableBarracudaOperators::*,
    FixedBarracudaOperators,
    VariableBarracudaOperators
};
use super::{
    ThreadContext,
    StackValue::*
};

use std::io::{Error, ErrorKind};
use statrs::function::erf::{erf, erfc, erfc_inv, erf_inv};
use libm::{ilogb, lgamma, tgamma};
use scilib::math::bessel;
use float_next_after::NextAfter;


pub struct BarracudaOperationExecutor {
    op: BarracudaOperators
}

impl BarracudaOperationExecutor {
    pub fn new(op: BarracudaOperators) -> Self {
        Self {
            op
        }
    }

    /// Executes relevant operation on a thread context.
    /// The enum will match and apply the appropriate "micro-code" of the operation
    /// The enum will match and apply the appropriate "micro-code" of the operation
    /// @context: The thread_context to apply the operation to
    /// @return: Ok() on success, relevant Error if the operation fails. If the operation
    /// is not implemented or unknown it will return ErrorKind::NotFound
    pub fn execute(&self, context: &mut ThreadContext) -> Result<(), Error> {
        match self.op {
            BarracudaOperators::FIXED(op) => {
                Self::execute_fixed_op(op, context)
            },
            BarracudaOperators::VARIABLE(op) => {
                Self::execute_variable_op(op, context)
            }
        }
    }

    pub fn execute_fixed_op(op: FixedBarracudaOperators, context: &mut ThreadContext) -> Result<(), Error> {
        match op {
            NULL => {
                Ok(())
            },
            ADD => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a + b))?)
            },
            SUB => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a - b))?)
            },
            MUL => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a * b))?)
            },
            DIV => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a / b))?)
            },
            AND => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a & b))?)
            },
            NAND => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(!(a & b)))?)
            },
            OR => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a | b))?)
            },
            NOR => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(!(a | b)))?)
            },
            XOR => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a ^ b))?)
            },
            NOT => {
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(!a))?)
            },
            INC => {
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a + 1))?)
            },
            DEC => {
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a - 1))?)
            },
            SWAP => {
                let b = context.pop()?;
                let a = context.pop()?;
                context.push(b)?;
                Ok(context.push(a)?)
            },
            DUP => {
                let a = context.pop()?;
                context.push(a)?;
                Ok(context.push(a)?)
            },
            OVER => {
                let b = context.pop()?;
                let a = context.pop()?;
                context.push(a)?;
                context.push(b)?;
                Ok(context.push(a)?)
            },
            DROP => {
                let _ = context.pop()?;
                Ok(())
            },
            LSHIFT => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a << b))?)
            },
            RSHIFT => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a >> b))?)
            },
            NEGATE => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(-a))?)
            },
            MALLOC => {
                let a = context.pop()?.into_u64() as usize;
                let region_address = context.heap.malloc(a)?;
                context.push(UINT(region_address as u64))
            },
            FREE => {
                let a = context.pop()?.into_u64() as usize;
                context.heap.free(a)
            },
            MEMCPY => {
                let c = context.pop()?.into_u64() as usize;
                let b = context.pop()?.into_u64() as usize;
                let a = context.pop()?.into_u64() as usize;
                context.heap.memcpy(a, b, c)
            },
            MEMSET => {
                let c = context.pop()?.into_u64() as usize;
                let b = context.pop()?.into_u64() as u8;
                let a = context.pop()?.into_u64() as usize;
                context.heap.memset(a, b, c)
            },
            READ => {
                let a = context.pop()?.into_u64() as usize;
                let value: f32 = context.heap.read_word(a)?;
                context.push(REAL(value as f64))
            },
            WRITE => {
                let b = context.pop()?.into_f64() as f32;
                let a = context.pop()?.into_u64() as usize;
                context.heap.write_word(a, b)?;
                Ok(())
            }
            ADD_PTR => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                Ok(context.push(INT(a + b))?)
            },
            SUB_PTR => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                Ok(context.push(INT(a - b))?)
            }
            TERNARY => {
                let c = context.pop()?;
                let b = context.pop()?;
                let a = context.pop()?.into_f64();
                if a > 0.0 {
                    Ok(context.push(b)?)
                } else {
                    Ok(context.push(c)?)
                }
            },
            EQ => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                context.push(UINT((a == b) as u64))
            },
            GT => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                context.push(UINT((a > b) as u64))
            },
            GTEQ => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                context.push(UINT((a >= b) as u64))
            },
            LT => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                context.push(UINT((a < b) as u64))
            },
            LTEQ => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                context.push(UINT((a <= b) as u64))
            },
            NEQ => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                context.push(UINT((a != b) as u64))
            },
            STK_READ => {
                let a = context.pop()?.into_u64() as usize;
                context.push(context.read_stack(a)?)
            },
            STK_WRITE => {
                let b = context.pop()?;
                let a = context.pop()?.into_u64() as usize;
                context.write_stack(a, b)
            },
            ACOS => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::acos(a)))?)
            },
            ACOSH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::acosh(a)))?)
            },
            ASIN => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::asin(a)))?)
            },
            ASINH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::asinh(a)))?)
            },
            ATAN => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::atan(a)))?)
            },
            ATAN2 => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::atan2(a, b)))?)
            },
            ATANH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::atanh(a)))?)
            },
            CBRT => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cbrt(a)))?)
            },
            CEIL => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::ceil(a)))?)
            },
            CPYSGN => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::copysign(a, b)))?)
            },
            COS => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cos(a)))?)
            },
            COSH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cosh(a)))?)
            },
            COSPI => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cos(a * std::f64::consts::PI)))?)
            },
            BESI0 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::i(a, 0).re))
            },
            BESI1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::i(a, 1).re))
            },
            ERF => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erf(a)))
            },
            ERFC => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erfc(a)))
            },
            ERFCI => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erfc_inv(a)))
            },
            ERFCX => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::exp(a * a) * erfc(a)))
            },
            ERFI => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erf_inv(a)))
            },
            EXP => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::exp(a)))
            },
            EXP10 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::powf(10.0, a)))
            },
            EXP2 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::powf(2.0, a)))
            },
            EXPM1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::exp_m1(a)))
            },
            FABS => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::abs(a)))
            },
            FDIM => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                if a > b {
                    context.push(REAL(a - b))
                } else {
                    context.push(REAL(0.0))
                }
            },
            FLOOR => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::floor(a)))
            },
            FMA => {
                let c = context.pop()?.into_f64();
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * b + c))
            },
            FMAX => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::max(a, b)))
            },
            FMIN => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::min(a, b)))
            },
            FMOD => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a % b))
            },
            FREXP => {
                let b = context.pop()?.into_u64() as usize;
                let a = context.pop()?.into_f64();
                let (significand, exponent) = libm::frexp(a);
                context.heap.write_word(b, exponent);
                context.push(REAL(significand)) // TODO(Connor): Check this is the expected output
            },
            HYPOT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::hypot(a, b)))
            },
            ILOGB => {
                let a = context.pop()?.into_f64();
                context.push(INT(ilogb(a) as i64))
            },
            ISFIN => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_finite(a) as i64))
            },
            ISINF => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_infinite(a) as i64))
            },
            ISNAN => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_nan(a) as i64))
            },
            BESJ0 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::j(a, 0).re))
            },
            BESJ1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::j(a, 1).re))
            },
            BESJN => {
                let b = context.pop()?.into_i64() as i32;
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::j(a, b).re))
            },
            LDEXP => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * f64::powf(2.0, b)))
            },
            LGAMMA => {
                let a = context.pop()?.into_f64();
                context.push(REAL(lgamma(a)))
            },
            LLRINT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::floor(a) as i64))
            },
            LLROUND => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::round(a) as i64))
            },
            LOG => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::ln(a)))
            },
            LOG10 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::log(a, 10.0)))
            },
            LOG1P => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::ln(1.0 + a)))
            },
            LOG2 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::log(a, 2.0)))
            },
            LOGB => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::log(a, b)))
            },
            LRINT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::floor(a) as i32 as i64))
            },
            LROUND => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::round(a) as i32 as i64))
            },
            MAX => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::max(a, 0.0))) // TODO(Connor): Review if this is the expected functionality
            },
            MIN => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::min(a, 1.0))) // TODO(Connor): Review if this is the expected functionality
            },
            MODF => {
                let b = context.pop()?.into_u64() as usize;
                let a = context.pop()?.into_f64();
                let (integer, fraction) = libm::modf(a);
                context.heap.write_word(b, integer);
                context.push(REAL(fraction)) // TODO(Connor): Review if this is the expected output
            },
            // NAN ? Check cuda functions
            // NEARINT ? Check cuda functions
            NXTAFT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a.next_after(b)))
            },
            // NORM ?
            // NORM3D ?
            // NORM4D ?
            // NORMCDF ? Single value for cdf?
            // NORMCDFINV ?
            POW => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(INT(f64::powf(a, b) as i64))
            },
            RCBRT => {
                let a = context.pop()?.into_f64();
                context.push(REAL(1.0 / f64::cbrt(a)))
            },
            REM => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                context.push(INT(a % b))
            },
            REMQUO => {
                let c = context.pop()?.into_u64() as usize;
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                let (remainder, quo) = libm::remquo(a, b);

                context.heap.write_word(c, quo)?;
                context.push(REAL(remainder))
            },
            RHYPOT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(1.0 / f64::hypot(a, b)))
            },
            RINT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::round(a) as i64))
            },
            // RNORM ?
            // RNORM3D ?
            // RNORM4D ?
            ROUND => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::round(a)))
            },
            RSQRT => {
                let a = context.pop()?.into_f64();
                context.push(REAL(1.0 / f64::sqrt(a)))
            },
            SCALBLN => { // TODO(Connor): This implementation is surely wrong but need more details
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * f64::powf(2.0, b)))
            },
            SCALBN => { // TODO(Connor): This implementation is surely wrong but need more details
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * f64::powf(2.0, b)))
            },
            SGNBIT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_sign_negative(a) as i64))
            },
            SIN => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sin(a)))
            },
            SINH => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sinh(a)))
            },
            SINPI => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sin(a * std::f64::consts::PI)))
            },
            SQRT => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sqrt(a)))
            },
            TAN => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::tan(a)))
            },
            TANH => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::tanh(a)))
            },
            TGAMMA => {
                let a = context.pop()?.into_f64();
                context.push(REAL(tgamma(a)))
            },
            TRUNC => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::trunc(a) as i64))
            },
            BESY0 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::y(a, 0).re))
            },
            BESY1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::y(a, 1).re))
            },
            BESYN => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::y(a, b).re))
            },
            PRINTC => {
                let a = context.pop()?.into_u64() as u8 as char;
                let mut out = context.output_handle.borrow_mut();
                write!(out, "{}", a)
            },
            PRINTCT => {
                let b = context.pop()?.into_u64() as u8 as char;
                let a = context.pop()?.into_u64();
                if a == context.thread_id {
                    let mut out = context.output_handle.borrow_mut();
                    write!(out, "{}", b)?;
                }
                Ok(())
            },
            PRINTFF => {
                // NOTE(LUKE) : TODO WHY DO I NEED THIS! (check other prints too)
                let a = context.pop()?.into_f64();
                //let a = context.pop()?.into_u64();
                let mut out = context.output_handle.borrow_mut();
                write!(out, "{}", a)
            },
            PRINTFFT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_u64();
                if a == context.thread_id {
                    let mut out = context.output_handle.borrow_mut();
                    write!(out, "{}", b)?;
                }
                Ok(())
            },
            LDPC => {
                context.push(UINT(context.get_pc() as u64))
            },
            LDTID => {
                context.push(UINT(context.thread_id))
            },
            LDSTK_PTR => {
                println!("LDSTK_PTR {}", context.get_stack_pointer().unwrap() as u64 + 1);
                context.push(UINT(context.get_stack_pointer().unwrap() as u64 + 1))
            },
            RCSTK_PTR => {
                let a = context.pop()?.into_u64() as usize - 1;
                println!("RCSTK_PTR {}", a);
                context.set_stack_pointer(a);
                Ok(())
            }

            _ => {
                let opcode: u32 = op.as_u32();
                Err(Error::new(ErrorKind::NotFound, format!("Unknown or unimplemented opcode used {:?}({:#06X})", op, opcode)))
            }
        }
    }

    pub fn execute_variable_op(op: VariableBarracudaOperators, context: &mut ThreadContext) -> Result<(), Error> {
        match op {
            LDNX(address) => {
                let value = context.get_env_var(address)?;
                context.push(REAL(value))
            }
            RCNX(address) => {
                let value = context.pop()?.into_f64();
                context.set_env_var(address, value)
            }
        }
    }
}