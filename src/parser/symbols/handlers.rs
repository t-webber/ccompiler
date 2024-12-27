use super::super::tree::binary::BinaryOperator;
use super::super::tree::node::Node;
use super::super::tree::unary::UnaryOperator;

pub fn handle_comma(current: &mut Node) -> Result<(), String> {
    if current
        .apply_to_last_list_initialiser(&|vec, _| vec.push(Node::Empty))
        .is_err()
    {
        current.push_op(BinaryOperator::Comma)?;
    }
    Ok(())
}

pub fn handle_double_binary(
    current: &mut Node,
    bin_op: BinaryOperator,
    un_op: UnaryOperator,
) -> Result<(), String> {
    current
        .push_op(bin_op)
        .map_or_else(|_| current.push_op(un_op), |()| Ok(()))
}

pub fn handle_double_unary(
    current: &mut Node,
    first: UnaryOperator,
    second: UnaryOperator,
) -> Result<(), String> {
    current
        .push_op(first)
        .map_or_else(|_| current.push_op(second), |()| Ok(()))
}
