#[cfg(test)]
mod test {
    use crate::model_objects::expressions::ArithExpression as AE;
    use crate::model_objects::expressions::BoolExpression as BE;
    use AE::Int;
    use BE::Bool;
    #[test]
    fn simplify_test_1() {
        let mut expr = (Bool(false) & BE::b_less_eq(Int(3), Int(2))) | Bool(true);
        expr.simplify();
        assert_eq!(Bool(true), expr);
    }

    #[test]
    fn simplify_test_2() {
        let mut expr = BE::b_less_eq(Int(2), Int(3));
        expr.simplify();
        assert_eq!(Bool(true), expr);
    }
}
