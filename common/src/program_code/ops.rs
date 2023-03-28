use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use strum_macros::EnumString;
use enum_assoc::Assoc;
use std::fmt;
use regex::Regex;
use std::str::FromStr;


/// BarracudaOperators is a enum of all the operations of the Barracuda VM.
/// Each enum is set to the associated opcode.
/// In general the operations will pop arguments of the computation stack in order of last_arg to
/// first_arg while popping and will push the result of the operation to the stack. This may not
/// be true for more complex functions such as MALLOC which will interact with the memory heap.
/// Refer to the opcode documentation for more information.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug,
         Eq, PartialEq,
         Copy, Clone,
         FromPrimitive, ToPrimitive, EnumString,
         Assoc)]
#[func(pub const fn consume(&self) -> i8)] // How many arguments does the operation consume
#[func(pub const fn produce(&self) -> i8)] // How many values does the operation generate
#[repr(u32)]
pub enum FixedBarracudaOperators {
    #[assoc(consume=0)]
    #[assoc(produce=0)]
    NULL       = 0x0000 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    STK_READ   = 0x0001,
    #[assoc(consume=2)]
    #[assoc(produce=0)]
    STK_WRITE  = 0x0002,

    #[assoc(consume=2)]
    #[assoc(produce=1)]
    ADD        = 0x03CC ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    SUB        = 0x03CD ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    MUL        = 0x03CE ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    DIV        = 0x03CF ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    AND        = 0x03D0 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    NAND       = 0x03D1 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    OR         = 0x03D2 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    NOR        = 0x03D3 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    XOR        = 0x03D4 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    NOT        = 0x03D5 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    INC        = 0x03D6 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    DEC        = 0x03D7 ,
    #[assoc(consume=2)]
    #[assoc(produce=2)]
    SWAP       = 0x03D8 ,
    #[assoc(consume=1)]
    #[assoc(produce=2)]
    DUP        = 0x03D9 ,
    #[assoc(consume=2)]
    #[assoc(produce=3)]
    OVER       = 0x03DA ,
    #[assoc(consume=1)]
    #[assoc(produce=0)]
    DROP       = 0x03DB ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    LSHIFT     = 0x03DC ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    RSHIFT     = 0x03DD ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    MALLOC     = 0x03DE ,
    #[assoc(consume=1)]
    #[assoc(produce=0)]
    FREE       = 0x03DF ,
    #[assoc(consume=3)]
    #[assoc(produce=0)]
    MEMCPY     = 0x03E0 ,
    #[assoc(consume=3)]
    #[assoc(produce=0)]
    MEMSET     = 0x03E1 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    READ       = 0x03E2 ,
    #[assoc(consume=2)]
    #[assoc(produce=0)]
    WRITE      = 0x03E3 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    ADD_PTR    = 0x03E4 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    SUB_PTR    = 0x03E5 ,
    #[assoc(consume=3)]
    #[assoc(produce=1)]
    TERNARY    = 0x03E6 ,

    #[assoc(consume=2)]
    #[assoc(produce=1)]
    EQ         = 0x03E7,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    GT         = 0x03E8,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    GTEQ       = 0x03E9,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    LT         = 0x03EA,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    LTEQ       = 0x03EB,

    // READ / WRITE OP CODES

    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ACOS       = 0x0798 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ACOSH      = 0x0799 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ASIN       = 0x079A ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ASINH      = 0x079B ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ATAN       = 0x079C ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    ATAN2      = 0x079D ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ATANH      = 0x079E ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    CBRT       = 0x079F ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    CEIL       = 0x07A0 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    CPYSGN     = 0x07A1 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    COS        = 0x07A2 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    COSH       = 0x07A3 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    COSPI      = 0x07A4 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    BESI0      = 0x07A5 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    BESI1      = 0x07A6 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ERF        = 0x07A7 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ERFC       = 0x07A8 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ERFCI      = 0x07A9 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ERFCX      = 0x07AA ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ERFI       = 0x07AB ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    EXP        = 0x07AC ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    EXP10      = 0x07AD ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    EXP2       = 0x07AE ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    EXPM1      = 0x07AF ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    FABS       = 0x07B0 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    FDIM       = 0x07B1 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    FLOOR      = 0x07B2 ,
    #[assoc(consume=3)]
    #[assoc(produce=1)]
    FMA        = 0x07B3 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    FMAX       = 0x07B4 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    FMIN       = 0x07B5 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    FMOD       = 0x07B6 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    FREXP      = 0x07B7 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    HYPOT      = 0x07B8 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ILOGB      = 0x07B9 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ISFIN      = 0x07BA ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ISINF      = 0x07BB ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ISNAN      = 0x07BC ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    BESJ0      = 0x07BD ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    BESJ1      = 0x07BE ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    BESJN      = 0x07BF ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    LDEXP      = 0x07C0 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LGAMMA     = 0x07C1 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LLRINT     = 0x07C2 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LLROUND    = 0x07C3 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LOG        = 0x07C4 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LOG10      = 0x07C5 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LOG1P      = 0x07C6 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LOG2       = 0x07C7 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    LOGB       = 0x07C8 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LRINT      = 0x07C9 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    LROUND     = 0x07CA ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    MAX        = 0x07CB ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    MIN        = 0x07CC ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    MODF       = 0x07CD ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NAN        = 0x07CE ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NEARINT    = 0x07CF ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    NXTAFT     = 0x07D0 ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NORM       = 0x07D1 ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NORM3D     = 0x07D2 ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NORM4D     = 0x07D3 ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NORMCDF    = 0x07D4 ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    NORMCDFINV = 0x07D5 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    POW        = 0x07D6 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    RCBRT      = 0x07D7 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    REM        = 0x07D8 ,
    #[assoc(consume=3)]
    #[assoc(produce=1)]
    REMQUO     = 0x07D9 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    RHYPOT     = 0x07DA ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    RINT       = 0x07DB ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    RNORM      = 0x07DC ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    RNORM3D    = 0x07DD ,
    #[assoc(consume=-1)]
    #[assoc(produce=-1)]
    RNORM4D    = 0x07DE ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    ROUND      = 0x07DF ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    RSQRT      = 0x07E0 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    SCALBLN    = 0x07E1 ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    SCALBN     = 0x07E2 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    SGNBIT     = 0x07E3 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    SIN        = 0x07E4 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    SINH       = 0x07E5 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    SINPI      = 0x07E6 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    SQRT       = 0x07E7 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    TAN        = 0x07E8 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    TANH       = 0x07E9 ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    TGAMMA     = 0x07EA ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    TRUNC      = 0x07EB ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    BESY0      = 0x07EC ,
    #[assoc(consume=1)]
    #[assoc(produce=1)]
    BESY1      = 0x07ED ,
    #[assoc(consume=2)]
    #[assoc(produce=1)]
    BESYN      = 0x07EE ,

    #[assoc(consume=1)]
    #[assoc(produce=0)]
    PRINTC     = 0x0B64 ,
    #[assoc(consume=2)]
    #[assoc(produce=0)]
    PRINTCT    = 0x0B65 ,
    #[assoc(consume=1)]
    #[assoc(produce=0)]
    PRINTFF    = 0x0B66 ,
    #[assoc(consume=2)]
    #[assoc(produce=0)]
    PRINTFFT   = 0x0B67 ,

    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDPC       = 0x12FC ,
    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDTID      = 0x12FD ,
    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDNXPTR    = 0x12FE ,

    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDSTK_PTR  = 0x12FF ,
    #[assoc(consume=1)]
    #[assoc(produce=0)]
    RCSTK_PTR  = 0x1300 ,

    // Lower (0) and upper (1) ranges for Load and Store into Nth variate userspace.
    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDNX0 = 0xF4240,

    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDNX1 = 0x4C4B40,

    #[assoc(consume=1)]
    #[assoc(produce=0)]
    RCNX0 = 0x4C4B41,

    #[assoc(consume=1)]
    #[assoc(produce=0)]
    RCNX1 = 0x895441
}

#[allow(dead_code)] // Used in library but not the binary
impl FixedBarracudaOperators {

    /// Converts opcode value into Operation enums.
    /// @opcode: Barracuda op code value.
    /// @returns Some(BarracudaOperator) representing opcode value, None otherwise
    pub fn from(opcode: u32) -> Option<Self> {
        FromPrimitive::from_u32(opcode)
    }

    /// Converts operator into value representing the opcode
    /// @returns: &self's representation as u32. This is not an option as all operators
    ///           have a valid u32 code.
    pub fn as_u32(&self) -> u32 {
        // Safe to unwrap here as enum should always map to an integer.
        self.to_u32().unwrap()
    }
}


#[derive(Debug,
Eq, PartialEq,
Copy, Clone, Assoc)]
#[func(pub const fn consume(&self) -> i8)] // How many arguments does the operation consume
#[func(pub const fn produce(&self) -> i8)] // How many values does the operation generate
pub enum VariableBarracudaOperators {
    // Load Nth Environment Variable
    #[assoc(consume=0)]
    #[assoc(produce=1)]
    LDNX(usize),

    // Write Nth Environment Variable
    #[assoc(consume=1)]
    #[assoc(produce=0)]
    RCNX(usize)
}

#[allow(dead_code)] // Used in library but not the binary
impl VariableBarracudaOperators {

    /// Converts opcode value into Operation enums.
    /// @opcode: Barracuda op code value.
    /// @returns Some(BarracudaOperator) representing opcode value, None otherwise
    pub fn from(opcode: u32) -> Option<Self> {
        if  opcode >= FixedBarracudaOperators::LDNX0.as_u32() &&
             opcode < FixedBarracudaOperators::LDNX1.as_u32() {
            let index = (opcode - FixedBarracudaOperators::LDNX0.as_u32()) as usize;
            return Some(Self::LDNX(index));
        }

        if  opcode >= FixedBarracudaOperators::RCNX0.as_u32() &&
             opcode < FixedBarracudaOperators::RCNX1.as_u32() {
            let index = (opcode - FixedBarracudaOperators::RCNX0.as_u32()) as usize;
            return Some(Self::RCNX(index));
        }

        return None;
    }

    /// Converts operator into value representing the opcode
    /// @returns: &self's representation as u32. This is not an option as all operators
    ///           have a valid u32 code.
    pub fn as_u32(&self) -> u32 {
         match self {
            VariableBarracudaOperators::LDNX(index) => {
                FixedBarracudaOperators::LDNX0.as_u32() + *index as u32
            }
            VariableBarracudaOperators::RCNX(index) => {
                FixedBarracudaOperators::RCNX0.as_u32() + *index as u32
            }
        }
    }
}

#[derive(Debug,
Eq, PartialEq,
Copy, Clone)]
pub enum BarracudaOperators {
    FIXED(FixedBarracudaOperators),
    VARIABLE(VariableBarracudaOperators)
}

impl BarracudaOperators {
    /// Converts opcode value into Operation enums.
    /// @opcode: Barracuda op code value.
    /// @returns Some(BarracudaOperator) representing opcode value, None otherwise
    pub fn from(opcode: u32) -> Option<Self> {
        if let Some(op) = FixedBarracudaOperators::from(opcode) {
            return Some(Self::FIXED(op));
        }

        if let Some(op) = VariableBarracudaOperators::from(opcode) {
            return Some(Self::VARIABLE(op));
        }

        return None
    }

    /// Converts operator into value representing the opcode
    /// @returns: &self's representation as u32. This is not an option as all operators
    ///           have a valid u32 code.
    pub fn as_u32(&self) -> u32 {
        match self {
            BarracudaOperators::FIXED(op) => {
                op.as_u32()
            }
            BarracudaOperators::VARIABLE(op) => {
                op.as_u32()
            }
        }
    }

    /// Returns the number of arguments the operation takes from the stack
    /// @return operation argument count, -1 if unknown, or indeterminate
    pub const fn consume(&self) -> i8 {
        match self {
            BarracudaOperators::FIXED(op) => {
                op.consume()
            }
            BarracudaOperators::VARIABLE(op) => {
                op.consume()
            }
        }
    }

    /// Returns the number of outputs the operation adds to the stack
    /// @return operation output count, -1 if unknown, or indeterminate
    pub const fn produce(&self) -> i8 {
        match self {
            BarracudaOperators::FIXED(op) => {
                op.produce()
            }
            BarracudaOperators::VARIABLE(op) => {
                op.produce()
            }
        }
    }
}

impl FromStr for VariableBarracudaOperators {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(?P<name>[A-Za-z]+)\((?P<args>\d+)\)").unwrap();
        let caps = re.captures(s).ok_or(())?;
        let name = &caps["name"];
        let args = &caps["args"];

        // NOTE: Matching using const strs right now however a more fleshed out solution relying on
        // the enum names would be perfered.
        match name {
            "LDNX" => {
                let arg_value = usize::from_str(args).map_err(|_| ())?;
                Ok(VariableBarracudaOperators::LDNX(arg_value))
            },
            "RCNX" => {
                let arg_value = usize::from_str(args).map_err(|_| ())?;
                Ok(VariableBarracudaOperators::RCNX(arg_value))
            }
            _ => {
                Err(())
            }
        }
    }
}

impl FromStr for BarracudaOperators {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fixed_result = FixedBarracudaOperators::from_str(s);
        let variable_result = VariableBarracudaOperators::from_str(s);

        return if let Ok(op) = fixed_result {
            Ok(BarracudaOperators::FIXED(op))
        } else if let Ok(op) = variable_result {
            Ok(BarracudaOperators::VARIABLE(op))
        } else {
            Err(())
        }
    }
}

impl fmt::Display for VariableBarracudaOperators {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LDNX(index) => {
                write!(f, "LDNX({})", index)
            }
            Self::RCNX(index) => {
                write!(f, "RCNX({})", index)
            }
        }
    }
}

impl fmt::Display for BarracudaOperators {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FIXED(op) => {
                write!(f, "{:?}", op)
            }
            Self::VARIABLE(op) => {
                write!(f, "{}", op)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BarracudaOperators::VARIABLE;
    use crate::VariableBarracudaOperators;
    use super::{BarracudaOperators, FixedBarracudaOperators};
    use super::BarracudaOperators::FIXED;
    use std::str::FromStr;

    #[test]
    fn test_from_str_fixed_op() {
        let test_str = "SUB_PTR";
        let op = BarracudaOperators::from_str(test_str)
            .expect("Could not parse string into fixed operator");
        assert_eq!(op, FIXED(FixedBarracudaOperators::SUB_PTR));
    }

    #[test]
    fn test_from_str_variable_op_ldnx() {
        let test_str = "LDNX(25)";
        let op = BarracudaOperators::from_str(test_str)
            .expect("Could not parse string into variable operator");
        assert_eq!(op, VARIABLE(VariableBarracudaOperators::LDNX(25)));
    }
}