use core::fmt;

use super::node::Node;
use super::{repr_option_node, Associativity, Operator};

#[derive(Debug, PartialEq)]
pub struct Unary {
    pub(super) arg: Option<Box<Node>>,
    pub(super) op: UnaryOperator,
}

#[allow(clippy::min_ident_chars)]
impl fmt::Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.op.associativity() == Associativity::LeftToRight {
            write!(f, "({}{})", repr_option_node(self.arg.as_ref()), self.op)
        } else {
            write!(f, "({}{})", self.op, repr_option_node(self.arg.as_ref()))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Address-of (`&`)
    AddressOf,
    BitwiseNot,
    /// Dereference (`*`)
    Indirection,
    LogicalNot,
    Minus,
    Plus,
    PostfixDecrement,
    PostfixIncrement,
    PrefixDecrement,
    PrefixIncrement,
}

impl Operator for UnaryOperator {
    fn associativity(&self) -> Associativity {
        match self {
            Self::PostfixIncrement | Self::PostfixDecrement => Associativity::LeftToRight,
            Self::PrefixIncrement
            | Self::PrefixDecrement
            | Self::Plus
            | Self::Minus
            | Self::BitwiseNot
            | Self::LogicalNot
            | Self::Indirection
            | Self::AddressOf => Associativity::RightToLeft,
        }
    }

    fn precedence(&self) -> u32 {
        match self {
            Self::PostfixIncrement | Self::PostfixDecrement => 1,
            Self::PrefixIncrement
            | Self::PrefixDecrement
            | Self::Plus
            | Self::Minus
            | Self::BitwiseNot
            | Self::LogicalNot
            | Self::Indirection
            | Self::AddressOf => 2,
        }
    }
}

#[allow(clippy::min_ident_chars)]
impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PostfixIncrement | Self::PrefixIncrement => "++",
                Self::PostfixDecrement | Self::PrefixDecrement => "--",
                Self::Plus => "+",
                Self::Minus => "-",
                Self::BitwiseNot => "~",
                Self::LogicalNot => "!",
                Self::Indirection => "*",
                Self::AddressOf => "&",
            }
        )
    }
}
