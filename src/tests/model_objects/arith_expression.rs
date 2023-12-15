#[cfg(test)]
mod test {
    use crate::model_objects::expressions::ArithExpression as AE;
    use AE::*;
    #[test]
    fn simplify_test_1() {
        let expr = AE::a_dif(Int(10), Int(5)); //10 - 5
        assert_eq!(Ok(Int(5)), expr.simplify());

        let expr = AE::a_add(Int(10), Int(5)); //10 + 5
        assert_eq!(Ok(Int(15)), expr.simplify());
    }

    #[test]
    fn simplify_test_2() {
        let expr = Multiplication(
            //(10 - 5) * (5 + 3)
            Box::new(AE::a_dif(Int(10), Int(5))),
            Box::new(AE::a_add(Int(5), Int(3))),
        );
        assert_eq!(Ok(Int(40)), expr.simplify());

        let expr = Multiplication(
            //(10 + 5) * (5 - 3)
            Box::new(AE::a_add(Int(10), Int(5))),
            Box::new(AE::a_dif(Int(5), Int(3))),
        );
        assert_eq!(Ok(Int(30)), expr.simplify());
    }
    #[test]
    fn simplify_test_3() {
        let expr = AE::a_dif(
            Clock(1),
            AE::a_dif(Int(5), AE::a_dif(Int(4), AE::a_dif(Int(3), Int(2)))), //5-(4-(3-2))
        );
        assert_eq!(Ok(AE::a_dif(Clock(1), Int(2))), expr.simplify());

        let expr = AE::a_add(
            Clock(1),
            AE::a_add(Int(5), AE::a_add(Int(4), AE::a_add(Int(3), Int(2)))),
        );
        assert_eq!(Ok(AE::a_add(Clock(1), Int(14))), expr.simplify());
    }
    #[test]
    fn simplify_test_4() {
        //((5-4)-3)-2
        let expr = AE::a_dif(
            Clock(1),
            AE::a_dif(AE::a_dif(AE::a_dif(Int(5), Int(4)), Int(3)), Int(2)),
        );
        assert_eq!(Ok(AE::a_dif(Clock(1), Int(-4))), expr.simplify());

        let expr = AE::a_add(
            Clock(1),
            AE::a_add(AE::a_add(AE::a_add(Int(5), Int(4)), Int(3)), Int(2)),
        );
        assert_eq!(Ok(AE::a_add(Clock(1), Int(14))), expr.simplify());
    }

    #[test]
    fn simplify_test_5() {
        //((5-4)-3)-2
        let expr = AE::a_dif(AE::a_dif(AE::a_dif(Int(5), Clock(4)), Int(3)), Int(2));
        assert_eq!(Ok(AE::a_dif(Int(0), Clock(4))), expr.simplify());

        let expr = AE::a_add(AE::a_add(AE::a_add(Int(5), Clock(4)), Int(3)), Int(2));
        assert_eq!(Ok(AE::a_add(Int(10), Clock(4))), expr.simplify());
    }

    #[test]
    fn simplify_test_6() {
        //5-(4-(3-2))
        let expr = AE::a_dif(Int(5), AE::a_dif(Int(4), AE::a_dif(Int(3), Clock(2))));
        assert_eq!(Ok(AE::a_dif(Int(4), Clock(2))), expr.simplify());

        let expr = AE::a_add(Int(5), AE::a_add(Int(4), AE::a_add(Int(3), Clock(2))));
        assert_eq!(Ok(AE::a_add(Int(12), Clock(2))), expr.simplify());
    }

    #[test]
    fn simplify_test_7() {
        //5-(4-(3-2))
        let expr = AE::a_dif(
            Int(5),
            AE::a_dif(Clock(4), AE::a_dif(Int(3), AE::a_dif(Int(2), Int(1)))),
        );
        assert_eq!(Ok(AE::a_dif(Clock(4), Int(3))), expr.simplify());

        let expr = AE::a_add(
            Int(5),
            AE::a_add(Clock(4), AE::a_add(Int(3), AE::a_add(Int(2), Int(1)))),
        );
        assert_eq!(Ok(AE::a_add(Clock(4), Int(11))), expr.simplify());
    }

    #[test]
    fn simplify_test_highoperators_ints() {
        let expr = AE::a_mul(Int(10), Int(5));
        assert_eq!(Ok(Int(50)), expr.simplify());

        let expr = AE::a_div(Int(10), Int(5));
        assert_eq!(Ok(Int(2)), expr.simplify());

        let expr = AE::a_mod(Int(10), Int(5));
        assert_eq!(Ok(Int(0)), expr.simplify());
    }

    #[test]
    fn simplify_test_highoperators_clocks() {
        let expr = AE::a_mul(Clock(10), Int(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::a_mul(Int(10), Clock(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::a_div(Clock(10), Int(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::a_div(Int(10), Clock(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::a_mod(Clock(10), Int(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::a_mod(Int(10), Clock(5));
        assert_eq!(expr.simplify().ok(), None);
    }
}
