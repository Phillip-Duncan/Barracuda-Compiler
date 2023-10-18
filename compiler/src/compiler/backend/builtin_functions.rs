use barracuda_common::FixedBarracudaOperators as OP;

pub static BARRACUDA_BUILT_IN_FUNCTIONS: &[OP] = &[
    OP::ACOS,
    OP::ACOSH,
    OP::ASIN,
    OP::ASINH,
    OP::ATAN,
    OP::ATAN2,
    OP::ATANH,
    OP::CBRT,
    OP::CEIL,
    OP::CPYSGN,
    OP::COS,
    OP::COSH,
    OP::COSPI,
    OP::BESI0,
    OP::BESI1,
    OP::ERF,
    OP::ERFC,
    OP::ERFCI,
    OP::ERFCX,
    OP::ERFI,
    OP::EXP,
    OP::EXP10,
    OP::EXP2,
    OP::EXPM1,
    OP::FABS,
    OP::FDIM,
    OP::FLOOR,
    OP::FMA,
    OP::FMAX,
    OP::FMIN,
    OP::FMOD,
    OP::FREXP,
    OP::HYPOT,
    OP::ILOGB,
    OP::ISFIN,
    OP::ISINF,
    OP::ISNAN,
    OP::BESJ0,
    OP::BESJ1,
    OP::BESJN,
    OP::LDEXP,
    OP::LGAMMA,
    OP::LLRINT,
    OP::LLROUND,
    OP::LOG,
    OP::LOG10,
    OP::LOG1P,
    OP::LOG2,
    OP::LOGB,
    OP::LRINT,
    OP::LROUND,
    OP::MAX,
    OP::MIN,
    OP::MODF,
    OP::NAN,
    OP::NEARINT,
    OP::NXTAFT,
    OP::NORM,
    OP::NORM3D,
    OP::NORM4D,
    OP::NORMCDF,
    OP::NORMCDFINV,
    OP::POW,
    OP::RCBRT,
    OP::REM,
    OP::REMQUO,
    OP::RHYPOT,
    OP::RINT,
    OP::RNORM,
    OP::RNORM3D,
    OP::RNORM4D,
    OP::ROUND,
    OP::RSQRT,
    OP::SCALBLN,
    OP::SCALBN,
    OP::SGNBIT,
    OP::SIN,
    OP::SINH,
    OP::SINPI,
    OP::SQRT,
    OP::TAN,
    OP::TANH,
    OP::TGAMMA,
    OP::TRUNC,
    OP::BESY0,
    OP::BESY1,
    OP::BESYN,
];