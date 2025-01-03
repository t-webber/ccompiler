//! Module to sort the keywords into different categories.

use super::super::types::Ast;
use super::super::types::literal::Literal;
use super::attributes::{AttributeKeyword as Attr, UnsortedAttributeKeyword as UnsortedAttr};
use super::control_flow::keyword::ControlFlowKeyword as CtrlFlow;
use super::functions::FunctionKeyword as Func;
use crate::lexer::api::Keyword;

/// Enum for the different types of keywords that exist.
pub enum KeywordParsing {
    /// Attribute keyword: applied on a variable
    Attr(Attr),
    /// Control flow keyword: `return`, `for`, `goto`, `case`, ...
    CtrlFlow(CtrlFlow),
    /// Boolean constant `false`
    False,
    /// Function keyword: `sizeof`, `static_assert`, ...
    Func(Func),
    /// `NULL` constant
    Nullptr,
    /// Boolean constant `true`
    True,
}

impl From<(Keyword, bool)> for KeywordParsing {
    fn from((keyword, case_context): (Keyword, bool)) -> Self {
        match keyword {
            // constants
            Keyword::True => Self::True,
            Keyword::False => Self::False,
            Keyword::Null | Keyword::Nullptr => Self::Nullptr,
            // functions
            Keyword::Sizeof => Self::Func(Func::Sizeof),
            Keyword::Typeof => Self::Func(Func::Typeof),
            Keyword::TypeofUnqual => Self::Func(Func::TypeofUnqual),
            Keyword::Alignof | Keyword::UAlignof => Self::Func(Func::Alignof),
            Keyword::StaticAssert | Keyword::UStaticAssert => Self::Func(Func::StaticAssert),
            // control flows
            Keyword::Do => Self::CtrlFlow(CtrlFlow::Do),
            Keyword::If => Self::CtrlFlow(CtrlFlow::If),
            Keyword::For => Self::CtrlFlow(CtrlFlow::For),
            Keyword::Case => Self::CtrlFlow(CtrlFlow::Case),
            Keyword::Else => Self::CtrlFlow(CtrlFlow::Else),
            Keyword::Goto => Self::CtrlFlow(CtrlFlow::Goto),
            Keyword::While => Self::CtrlFlow(CtrlFlow::While),
            Keyword::Break => Self::CtrlFlow(CtrlFlow::Break),
            Keyword::Return => Self::CtrlFlow(CtrlFlow::Return),
            Keyword::Switch => Self::CtrlFlow(CtrlFlow::Switch),
            Keyword::Continue => Self::CtrlFlow(CtrlFlow::Continue),
            Keyword::Default if case_context => Self::CtrlFlow(CtrlFlow::Default),
            // user-defined types
            Keyword::Enum => Self::CtrlFlow(CtrlFlow::Enum),
            Keyword::Union => Self::CtrlFlow(CtrlFlow::Union),
            Keyword::Struct => Self::CtrlFlow(CtrlFlow::Struct),
            Keyword::Typedef => Self::CtrlFlow(CtrlFlow::Typedef),
            // attributes
            Keyword::Int => Self::Attr(Attr::from(UnsortedAttr::Int)),
            Keyword::Long => Self::Attr(Attr::from(UnsortedAttr::Long)),
            Keyword::Auto => Self::Attr(Attr::from(UnsortedAttr::Auto)),
            Keyword::Char => Self::Attr(Attr::from(UnsortedAttr::Char)),
            Keyword::Void => Self::Attr(Attr::from(UnsortedAttr::Void)),
            Keyword::Short => Self::Attr(Attr::from(UnsortedAttr::Short)),
            Keyword::Float => Self::Attr(Attr::from(UnsortedAttr::Float)),
            Keyword::Const => Self::Attr(Attr::from(UnsortedAttr::Const)),
            Keyword::Inline => Self::Attr(Attr::from(UnsortedAttr::Inline)),
            Keyword::Double => Self::Attr(Attr::from(UnsortedAttr::Double)),
            Keyword::Signed => Self::Attr(Attr::from(UnsortedAttr::Signed)),
            Keyword::Extern => Self::Attr(Attr::from(UnsortedAttr::Extern)),
            Keyword::Static => Self::Attr(Attr::from(UnsortedAttr::Static)),
            Keyword::UAtomic => Self::Attr(Attr::from(UnsortedAttr::UAtomic)),
            Keyword::UBigInt => Self::Attr(Attr::from(UnsortedAttr::UBigInt)),
            Keyword::Default => Self::Attr(Attr::from(UnsortedAttr::Default)),
            Keyword::Unsigned => Self::Attr(Attr::from(UnsortedAttr::Unsigned)),
            Keyword::Register => Self::Attr(Attr::from(UnsortedAttr::Register)),
            Keyword::Restrict => Self::Attr(Attr::from(UnsortedAttr::Restrict)),
            Keyword::Volatile => Self::Attr(Attr::from(UnsortedAttr::Volatile)),
            Keyword::UComplex => Self::Attr(Attr::from(UnsortedAttr::UComplex)),
            Keyword::UGeneric => Self::Attr(Attr::from(UnsortedAttr::UGeneric)),
            Keyword::UNoreturn => Self::Attr(Attr::from(UnsortedAttr::UNoreturn)),
            Keyword::Constexpr => Self::Attr(Attr::from(UnsortedAttr::Constexpr)),
            Keyword::UDecimal64 => Self::Attr(Attr::from(UnsortedAttr::UDecimal64)),
            Keyword::UImaginary => Self::Attr(Attr::from(UnsortedAttr::UImaginary)),
            Keyword::UDecimal32 => Self::Attr(Attr::from(UnsortedAttr::UDecimal32)),
            Keyword::UDecimal128 => Self::Attr(Attr::from(UnsortedAttr::UDecimal128)),
            Keyword::Alignas | Keyword::UAlignas => Self::Attr(Attr::from(UnsortedAttr::Alignas)),
            Keyword::Bool | Keyword::UBool => Self::Attr(Attr::from(UnsortedAttr::Bool)),
            Keyword::ThreadLocal | Keyword::UThreadLocal => {
                Self::Attr(Attr::from(UnsortedAttr::ThreadLocal))
            }
        }
    }
}

impl PushInNode for KeywordParsing {
    fn push_in_node(self, node: &mut Ast) -> Result<(), String> {
        match self {
            Self::Func(func) => func.push_in_node(node),
            Self::Attr(attr) => attr.push_in_node(node),
            Self::CtrlFlow(ctrl) => ctrl.push_in_node(node),
            Self::Nullptr => node.push_block_as_leaf(Ast::Leaf(Literal::Nullptr)),
            Self::True => node.push_block_as_leaf(Ast::Leaf(Literal::ConstantBool(true))),
            Self::False => node.push_block_as_leaf(Ast::Leaf(Literal::ConstantBool(false))),
        }
    }
}

/// Trait to push a keyword inside a current [`Ast`].
pub trait PushInNode {
    /// Function to push a keyword inside a current [`Ast`].
    fn push_in_node(self, node: &mut Ast) -> Result<(), String>;
}
