mod add;
mod and;
mod cal;
mod dec;
mod hlt;
mod inc;
mod jeq;
mod jge;
mod jgt;
mod jle;
mod jlt;
mod jne;
mod lsh;
mod mov;
mod mul;
mod not;
mod or;
mod pop;
mod psh;
mod ret;
mod rsh;
mod sub;
mod xor;

#[rustfmt::skip]
use nom::branch::alt;
#[rustfmt::skip]
use nom::IResult;

use add::parse_add;
use and::parse_and;
use cal::parse_cal;
use dec::parse_dec;
use hlt::parse_hlt;
use inc::parse_inc;
use jeq::parse_jeq;
use jge::parse_jge;
use jgt::parse_jgt;
use jle::parse_jle;
use jlt::parse_jlt;
use jne::parse_jne;
use lsh::parse_lsh;
use mov::parse_mov;
use mul::parse_mul;
use not::parse_not;
use or::parse_or;
use pop::parse_pop;
use psh::parse_psh;
use ret::parse_ret;
use rsh::parse_rsh;
use sub::parse_sub;
use xor::parse_xor;

pub use crate::types::{Atom, Instruction, InstructionKind};

pub fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    // alt is only implemented for tuples of size up to 21
    alt((
        parse_jeq,
        parse_hlt,
        alt((
            parse_add, parse_and, parse_cal, parse_dec, parse_inc, parse_jge, parse_jgt, parse_jle, parse_jlt,
            parse_jne, parse_lsh, parse_mov, parse_mul, parse_not, parse_or, parse_pop, parse_psh, parse_rsh,
            parse_sub, parse_xor, parse_ret,
        )),
    ))(input)
}
