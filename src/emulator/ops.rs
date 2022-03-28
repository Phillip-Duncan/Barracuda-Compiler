use crate::emulator::{ThreadContext, StackValue::*};
use std::io::{Error, ErrorKind};
use statrs::function::erf::{erf, erfc, erfc_inv, erf_inv};
use libm::{ilogb, lgamma, tgamma};
use scilib::math::bessel;
use float_next_after::NextAfter;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

/// MathStackOperators is a enum of all the operations of the MathStack VM.
/// Each enum is set to the associated opcode.
/// Every enum has the .execute(context: ThreadContext) function. Which will
/// mutate the stack with the relevant instruction.
/// In general the operations will pop arguments of the computation stack in order of last_arg to
/// first_arg while popping and will push the result of the operation to the stack. This may not
/// be true for more complex functions such as MALLOC which will interact with the memory heap.
/// Refer to the opcode documentation for more information.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, FromPrimitive)]
#[repr(u32)]
pub enum MathStackOperators {
    NULL       = 0x0000 , 
    ADD        = 0x03CC , 
    SUB        = 0x03CD , 
    MUL        = 0x03CE , 
    DIV        = 0x03CF , 
    AND        = 0x03D0 , 
    NAND       = 0x03D1 , 
    OR         = 0x03D2 , 
    NOR        = 0x03D3 , 
    XOR        = 0x03D4 , 
    NOT        = 0x03D5 , 
    INC        = 0x03D6 , 
    DEC        = 0x03D7 , 
    SWAP       = 0x03D8 , 
    DUP        = 0x03D9 , 
    OVER       = 0x03DA , 
    DROP       = 0x03DB , 
    LSHIFT     = 0x03DC , 
    RSHIFT     = 0x03DD , 
    MALLOC     = 0x03DE , 
    FREE       = 0x03DF , 
    MEMCPY     = 0x03E0 , 
    MEMSET     = 0x03E1 , 
    READ       = 0x03E2 , 
    WRITE      = 0x03E3 , 
    ADD_PTR    = 0x03E4 , 
    SUB_PTR    = 0x03E5 , 
    TERNARY    = 0x03E6 ,

    ACOS       = 0x0798 , 
    ACOSH      = 0x0799 , 
    ASIN       = 0x079A , 
    ASINH      = 0x079B , 
    ATAN       = 0x079C , 
    ATAN2      = 0x079D , 
    ATANH      = 0x079E , 
    CBRT       = 0x079F , 
    CEIL       = 0x07A0 ,
    CPYSGN     = 0x07A1 , 
    COS        = 0x07A2 , 
    COSH       = 0x07A3 , 
    COSPI      = 0x07A4 , 
    BESI0      = 0x07A5 , 
    BESI1      = 0x07A6 , 
    ERF        = 0x07A7 , 
    ERFC       = 0x07A8 , 
    ERFCI      = 0x07A9 , 
    ERFCX      = 0x07AA , 
    ERFI       = 0x07AB , 
    EXP        = 0x07AC , 
    EXP10      = 0x07AD , 
    EXP2       = 0x07AE , 
    EXPM1      = 0x07AF , 
    FABS       = 0x07B0 , 
    FDIM       = 0x07B1 , 
    FLOOR      = 0x07B2 , 
    FMA        = 0x07B3 , 
    FMAX       = 0x07B4 , 
    FMIN       = 0x07B5 , 
    FMOD       = 0x07B6 , 
    FREXP      = 0x07B7 , 
    HYPOT      = 0x07B8 , 
    ILOGB      = 0x07B9 , 
    ISFIN      = 0x07BA , 
    ISINF      = 0x07BB , 
    ISNAN      = 0x07BC , 
    BESJ0      = 0x07BD , 
    BESJ1      = 0x07BE , 
    BESJN      = 0x07BF , 
    LDEXP      = 0x07C0 , 
    LGAMMA     = 0x07C1 , 
    LLRINT     = 0x07C2 , 
    LLROUND    = 0x07C3 , 
    LOG        = 0x07C4 , 
    LOG10      = 0x07C5 , 
    LOG1P      = 0x07C6 , 
    LOG2       = 0x07C7 , 
    LOGB       = 0x07C8 , 
    LRINT      = 0x07C9 , 
    LROUND     = 0x07CA , 
    MAX        = 0x07CB , 
    MIN        = 0x07CC , 
    MODF       = 0x07CD , 
    NAN        = 0x07CE , 
    NEARINT    = 0x07CF , 
    NXTAFT     = 0x07D0 , 
    NORM       = 0x07D1 , 
    NORM3D     = 0x07D2 , 
    NORM4D     = 0x07D3 , 
    NORMCDF    = 0x07D4 , 
    NORMCDFINV = 0x07D5 , 
    POW        = 0x07D6 , 
    RCBRT      = 0x07D7 , 
    REM        = 0x07D8 , 
    REMQUO     = 0x07D9 , 
    RHYPOT     = 0x07DA , 
    RINT       = 0x07DB , 
    RNORM      = 0x07DC , 
    RNORM3D    = 0x07DD , 
    RNORM4D    = 0x07DE , 
    ROUND      = 0x07DF , 
    RSQRT      = 0x07E0 , 
    SCALBLN    = 0x07E1 , 
    SCALBN     = 0x07E2 , 
    SGNBIT     = 0x07E3 , 
    SIN        = 0x07E4 , 
    SINH       = 0x07E5 , 
    SINPI      = 0x07E6 , 
    SQRT       = 0x07E7 , 
    TAN        = 0x07E8 , 
    TANH       = 0x07E9 , 
    TGAMMA     = 0x07EA , 
    TRUNC      = 0x07EB , 
    BESY0      = 0x07EC , 
    BESY1      = 0x07ED , 
    BESYN      = 0x07EE ,

    PRINTC     = 0x0B64 , 
    PRINTCT    = 0x0B65 , 
    PRINTFF    = 0x0B66 , 
    PRINTFFT   = 0x0B67 ,

    LDA        = 0x12FC , 
    LDB        = 0x12FD , 
    LDC        = 0x12FE , 
    LDD        = 0x12FF , 
    LDE        = 0x1300 , 
    LDF        = 0x1301 , 
    LDG        = 0x1302 , 
    LDH        = 0x1303 , 
    LDI        = 0x1304 , 
    LDJ        = 0x1305 , 
    LDK        = 0x1306 , 
    LDL        = 0x1307 , 
    LDM        = 0x1308 , 
    LDN        = 0x1309 , 
    LDO        = 0x130A , 
    LDP        = 0x130B , 
    LDQ        = 0x130C , 
    LDR        = 0x130D , 
    LDS        = 0x130E , 
    LDT        = 0x130F , 
    LDU        = 0x1310 , 
    LDV        = 0x1311 , 
    LDW        = 0x1312 , 
    LDX        = 0x1313 , 
    LDY        = 0x1314 , 
    LDZ        = 0x1315 , 
    LDDX       = 0x1316 , 
    LDDY       = 0x1317 , 
    LDDZ       = 0x1318 , 
    LDDT       = 0x1319 , 
    LDA0       = 0x131A , 
    LDB0       = 0x131B , 
    LDC0       = 0x131C , 
    LDD0       = 0x131D , 
    LDE0       = 0x131E , 
    LDF0       = 0x131F , 
    LDG0       = 0x1320 , 
    LDH0       = 0x1321 , 
    LDI0       = 0x1322 , 
    LDJ0       = 0x1323 , 
    LDK0       = 0x1324 , 
    LDL0       = 0x1325 , 
    LDM0       = 0x1326 , 
    LDN0       = 0x1327 , 
    LDO0       = 0x1328 , 
    LDP0       = 0x1329 , 
    LDQ0       = 0x132A , 
    LDR0       = 0x132B , 
    LDS0       = 0x132C , 
    LDT0       = 0x132D , 
    LDU0       = 0x132E , 
    LDV0       = 0x132F , 
    LDW0       = 0x1330 , 
    LDX0       = 0x1331 , 
    LDY0       = 0x1332 , 
    LDZ0       = 0x1333 ,

    LDPC       = 0x1334 , 
    LDTID      = 0x1335 ,

    RCA        = 0x16C8 , 
    RCB        = 0x16C9 , 
    RCC        = 0x16CA , 
    RCD        = 0x16CB , 
    RCE        = 0x16CC , 
    RCF        = 0x16CD , 
    RCG        = 0x16CE , 
    RCH        = 0x16CF , 
    RCI        = 0x16D0 , 
    RCJ        = 0x16D1 , 
    RCK        = 0x16D2 , 
    RCL        = 0x16D3 , 
    RCM        = 0x16D4 , 
    RCN        = 0x16D5 , 
    RCO        = 0x16D6 , 
    RCP        = 0x16D7 , 
    RCQ        = 0x16D8 , 
    RCR        = 0x16D9 , 
    RCS        = 0x16DA , 
    RCT        = 0x16DB , 
    RCU        = 0x16DC , 
    RCV        = 0x16DD , 
    RCW        = 0x16DE , 
    RCX        = 0x16DF , 
    RCY        = 0x16E0 , 
    RCZ        = 0x16E1 , 
    RCDX       = 0x16E2 , 
    RCDY       = 0x16E3 , 
    RCDZ       = 0x16E4 , 
    RCDT       = 0x16E5
}

impl MathStackOperators {

    /// Converts opcode value into Operation enums.
    /// @opcode: MathStack op code value.
    /// @returns Some(MathStackOperator) representing opcode value, None otherwise
    pub(crate) fn from(opcode: u32) -> Option<Self> {
        FromPrimitive::from_u32(opcode)
    }

    /// Executes relevant operation on a thread context.
    /// The enum will match and apply the appropriate "micro-code" of the operation
    /// The enum will match and apply the appropriate "micro-code" of the operation
    /// @context: The thread_context to apply the operation to
    /// @return: Ok() on success, relevant Error if the operation fails. If the operation
    /// is not implemented or unknown it will return ErrorKind::NotFound
    pub(crate) fn execute(&self, context: &mut ThreadContext) -> Result<(), Error> {
        match self {
            Self::NULL => {
                Ok(())
            },
            Self::ADD => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a+b))?)
            },
            Self::SUB => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a-b))?)
            },
            Self::MUL => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a*b))?)
            },
            Self::DIV => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(a/b))?)
            },
            Self::AND => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a & b))?)
            },
            Self::NAND => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(!(a & b)))?)
            },
            Self::OR => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a | b))?)
            },
            Self::NOR => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(!(a | b)))?)
            },
            Self::XOR => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a ^ b))?)
            },
            Self::NOT => {
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(!a))?)
            },
            Self::INC => {
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a + 1))?)
            },
            Self::DEC => {
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a - 1))?)
            },
            Self::SWAP => {
                let b = context.pop()?;
                let a = context.pop()?;
                context.push(b)?;
                Ok(context.push(a)?)
            },
            Self::DUP => {
                let a = context.pop()?;
                context.push(a)?;
                Ok(context.push(a)?)
            },
            Self::OVER => {
                if context.stack.len() > 0 {
                    context.push(context.stack[0])
                } else {
                    Ok(()) // TODO(Connor): Review if the behaviour should be an error or ignored.
                }
            },
            Self::DROP => {
                let _ = context.pop()?;
                Ok(())
            },
            Self::LSHIFT => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a << b))?)
            },
            Self::RSHIFT => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_u64();
                Ok(context.push(UINT(a >> b))?)
            },
            Self::MALLOC => {
                let a = context.pop()?.into_u64() as usize;
                let region_address = context.heap.malloc(a)?;
                context.push(UINT(region_address as u64))
            },
            Self::FREE => {
                let a = context.pop()?.into_u64() as usize;
                context.heap.free(a)
            },
            Self::MEMCPY => {
                let c = context.pop()?.into_u64() as usize;
                let b = context.pop()?.into_u64() as usize;
                let a = context.pop()?.into_u64() as usize;
                context.heap.memcpy(a, b, c)
            },
            Self::MEMSET => {
                let c = context.pop()?.into_u64() as usize;
                let b = context.pop()?.into_u64() as u8;
                let a = context.pop()?.into_u64() as usize;
                context.heap.memset(a, b, c)
            },
            Self::READ => {
                let a = context.pop()?.into_u64() as usize;
                let value = context.heap.read(a)?;
                context.push(UINT(value as u64))
            },
            Self::WRITE => {
                let b = context.pop()?.into_u64() as u8;
                let a = context.pop()?.into_u64() as usize;
                context.heap.write(a, b)
            }
            Self::ADD_PTR => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                Ok(context.push(INT(a + b))?)
            },
            Self::SUB_PTR => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                Ok(context.push(INT(a - b))?)
            }
            Self::TERNARY => {
                let c = context.pop()?;
                let b = context.pop()?;
                let a = context.pop()?.into_f64();
                if a > 0.0 {
                    Ok(context.push(b)?)
                } else {
                    Ok(context.push(c)?)
                }
            },
            Self::ACOS => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::acos(a)))?)
            },
            Self::ACOSH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::acosh(a)))?)
            },
            Self::ASIN => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::asin(a)))?)
            },
            Self::ASINH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::asinh(a)))?)
            },
            Self::ATAN => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::atan(a)))?)
            },
            Self::ATAN2 => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::atan2(a, b)))?)
            },
            Self::ATANH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::atanh(a)))?)
            },
            Self::CBRT => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cbrt(a)))?)
            },
            Self::CEIL => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::ceil(a)))?)
            },
            Self::CPYSGN => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::copysign(a, b)))?)
            },
            Self::COS => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cos(a)))?)
            },
            Self::COSH => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cosh(a)))?)
            },
            Self::COSPI => {
                let a = context.pop()?.into_f64();
                Ok(context.push(REAL(f64::cos(a * std::f64::consts::PI)))?)
            },
            Self::BESI0 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::i(a, 0).re))
            },
            Self::BESI1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::i(a, 1).re))
            },
            Self::ERF => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erf(a)))
            },
            Self::ERFC => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erfc(a)))
            },
            Self::ERFCI => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erfc_inv(a)))
            },
            Self::ERFCX => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::exp(a*a) * erfc(a)))
            },
            Self::ERFI => {
                let a = context.pop()?.into_f64();
                context.push(REAL(erf_inv(a)))
            },
            Self::EXP => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::exp(a)))
            },
            Self::EXP10 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::powf(10.0, a)))
            },
            Self::EXP2 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::powf(2.0, a)))
            },
            Self::EXPM1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::exp_m1(a)))
            },
            Self::FABS => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::abs(a)))
            },
            Self::FDIM => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                if a > b {
                    context.push(REAL(a-b))
                } else {
                    context.push(REAL(0.0))
                }
            },
            Self::FLOOR => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::floor(a)))
            },
            Self::FMA => {
                let c = context.pop()?.into_f64();
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * b + c))
            },
            Self::FMAX => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::max(a, b)))
            },
            Self::FMIN => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::min(a, b)))
            },
            Self::FMOD => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a % b))
            },
            Self::FREXP => {
                let b = context.pop()?.into_u64() as usize;
                let a = context.pop()?.into_f64();
                let (significand, exponent) = libm::frexp(a);
                context.heap.write_word(b, exponent);
                context.push(REAL(significand)) // TODO(Connor): Check this is the expected output
            },
            Self::HYPOT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::hypot(a,b)))
            },
            Self::ILOGB => {
                let a = context.pop()?.into_f64();
                context.push(INT(ilogb(a) as i64))
            },
            Self::ISFIN => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_finite(a) as i64))
            },
            Self::ISINF => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_infinite(a) as i64))
            },
            Self::ISNAN => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_nan(a) as i64))
            },
            Self::BESJ0 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::j(a, 0).re))
            },
            Self::BESJ1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::j(a, 1).re))
            },
            Self::BESJN => {
                let b = context.pop()?.into_i64() as i32;
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::j(a, b).re))
            },
            Self::LDEXP => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * f64::powf(2.0, b)))
            },
            Self::LGAMMA => {
                let a = context.pop()?.into_f64();
                context.push(REAL(lgamma(a)))
            },
            Self::LLRINT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::floor(a) as i64))
            },
            Self::LLROUND => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::round(a) as i64))
            },
            Self::LOG => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::ln(a)))
            },
            Self::LOG10 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::log(a, 10.0)))
            },
            Self::LOG1P => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::ln(1.0+a)))
            },
            Self::LOG2 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::log(a, 2.0)))
            },
            Self::LOGB => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::log(a, b)))
            },
            Self::LRINT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::floor(a) as i32 as i64))
            },
            Self::LROUND => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::round(a) as i32 as i64))
            },
            Self::MAX => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::max(a, 0.0))) // TODO(Connor): Review if this is the expected functionality
            },
            Self::MIN => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::min(a, 1.0))) // TODO(Connor): Review if this is the expected functionality
            },
            Self::MODF => {
                let b = context.pop()?.into_u64() as usize;
                let a = context.pop()?.into_f64();
                let (integer, fraction) = libm::modf(a);
                context.heap.write_word(b, integer);
                context.push(REAL(fraction)) // TODO(Connor): Review if this is the expected output
            },
            // NAN ?
            // NEARINT ?
            Self::NXTAFT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a.next_after(b)))
            },
            // NORM ?
            // NORM3D ?
            // NORM4D ?
            // NORMCDF ? Single value for cdf?
            // NORMCDFINV ?
            Self::POW => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(INT(f64::powf(a,b) as i64))
            },
            Self::RCBRT => {
                let a = context.pop()?.into_f64();
                context.push(REAL(1.0/f64::cbrt(a)))
            },
            Self::REM => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                context.push(INT(a % b))
            },
            Self::REMQUO => {
                let c = context.pop()?.into_u64() as usize;
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                let (remainder, quo) = libm::remquo(a,b);

                context.heap.write_word(c, quo)?;
                context.push(REAL(remainder))
            },
            Self::RHYPOT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(1.0/f64::hypot(a,b)))
            },
            Self::RINT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::round(a) as i64))
            },
            // RNORM ?
            // RNORM3D ?
            // RNORM4D ?
            Self::ROUND => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::round(a)))
            },
            Self::RSQRT => {
                let a = context.pop()?.into_f64();
                context.push(REAL(1.0 / f64::sqrt(a)))
            },
            Self::SCALBLN => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * u64::pow(2, b as u32)))
            },
            Self::SCALBN => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_f64();
                context.push(REAL(a * u64::pow(2, b as u32)))
            },
            Self::SGNBIT => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::is_sign_negative(a) as i64))
            },
            Self::SIN => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sin(a)))
            },
            Self::SINH => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sinh(a)))
            },
            Self::SINPI => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sin(a * std::f64::consts::PI)))
            },
            Self::SQRT => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::sqrt(a)))
            },
            Self::TAN => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::tan(a)))
            },
            Self::TANH => {
                let a = context.pop()?.into_f64();
                context.push(REAL(f64::tanh(a)))
            },
            Self::TGAMMA => {
                let a = context.pop()?.into_f64();
                context.push(REAL(tgamma(a)))
            },
            Self::TRUNC => {
                let a = context.pop()?.into_f64();
                context.push(INT(f64::trunc(a) as i64))
            },
            Self::BESY0 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::y(a, 0).re))
            },
            Self::BESY1 => {
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::y(a, 1).re))
            },
            Self::BESYN => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_f64();
                context.push(REAL(bessel::y(a, b).re))
            },
            Self::PRINTC => {
                let a = context.pop()?.into_u64() as u8 as char;
                let mut out = context.output_handle.borrow_mut();
                write!(out, "{}", a)
            },
            Self::PRINTCT => {
                let b = context.pop()?.into_u64() as u8 as char;
                let a = context.pop()?.into_u64();
                if a == context.thread_id {
                    let mut out = context.output_handle.borrow_mut();
                    write!(out, "{}", b)?;
                }
                Ok(())
            },
            Self::PRINTFF => {
                let a = context.pop()?.into_f64();
                let mut out = context.output_handle.borrow_mut();
                write!(out, "{}", a)
            },
            Self::PRINTFFT => {
                let b = context.pop()?.into_f64();
                let a = context.pop()?.into_u64();
                if a == context.thread_id {
                    let mut out = context.output_handle.borrow_mut();
                    write!(out, "{}", b)?;
                }
                Ok(())
            },
            Self::LDTID => {
                context.push(UINT(context.thread_id))
            },
            _ => {
                // Match by value if not found above (This simplifies loading env vars)
                let opcode: u32 = *self as u32;

                // Load env variable
                if opcode >= Self::LDA as u32 && opcode <= Self::LDZ0 as u32 {
                    let env_var_index = (opcode - Self::LDA as u32) as usize;
                    let env_var = context.env_vars[env_var_index];
                    Ok(context.push(REAL(env_var))?)

                // Save env variable
                } else if opcode >= Self::RCA as u32 && opcode <= Self::RCZ as u32 {
                    let env_var_index = (opcode - Self::RCA as u32) as usize;
                    let env_var = context.pop()?.into_f64();
                    context.env_vars[env_var_index] = env_var;
                    Ok(())

                // Unknown Opcode
                } else {
                    Err(Error::new(ErrorKind::NotFound, format!("Unknown or unimplemented opcode used {:?}({:#06X})", self, opcode)))
                }
            }
        }
    }
}
