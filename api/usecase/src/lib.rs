pub mod route;
pub mod user;

#[cfg(test)]
mod test_macros {
    #[macro_export]
    macro_rules! expect_once {
        ($mock_expr:expr, $method_name:ident) => {
            concat_idents::concat_idents!(expect_method = expect_, $method_name {
                $mock_expr.expect_method().once()
            })
        };
        (private $api_exp:expr, $method_name:ident, $in_exp:expr) => {
            expect_once!($api_exp, $method_name)
                .withf(move |input| {
                    assert_eq!(*input, $in_exp);
                    true
                })
        };
        // TODO: 繰り返しで共通化する方法を探る（Closureの引数がうまくいかない）
        (private $api_exp:expr, $method_name:ident, $in_exp0:expr, $in_exp1:expr) => {
            expect_once!($api_exp, $method_name)
                .withf(move |input0, input1| {
                    assert_eq!(*input0, $in_exp0);
                    assert_eq!(*input1, $in_exp1);
                    true
                })
        };
        ($api_exp:expr, $method_name:ident, $in_exp:expr, $out_exp:expr) => {
            expect_once!(private $api_exp, $method_name, $in_exp)
                .return_const(Ok($out_exp))
        };
        ($api_exp:expr, $method_name:ident, $in_exp0:expr, $in_exp1:expr, $out_exp:expr) => {
            expect_once!(private $api_exp, $method_name, $in_exp0, $in_exp1)
                .return_const(Ok($out_exp))
        };
        ($api_exp:expr, $method_name:ident, $in_exp:expr => $out_exp:expr) => {
            expect_once!(private $api_exp, $method_name, $in_exp)
                .returning(move |mut_input| {
                    *mut_input = $out_exp.clone();
                    Ok(())
                })
        };
    }

    #[macro_export]
    macro_rules! expect_at_repository {
        ($usecase:expr, $method_name:ident, $out_exp:expr) => {
            crate::expect_once!($usecase.repository, $method_name).return_const(Ok($out_exp))
        };
        ($usecase:expr, $method_name:ident, $in_exp:expr, $out_exp:expr) => {
            expect_at_repository!($usecase, $method_name, $out_exp).withf(move |input, _| {
                assert_eq!(*input, $in_exp);
                true
            })
        };
    }
}
