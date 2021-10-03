pub mod route;

#[cfg(test)]
mod test_macros {
    #[macro_export]
    macro_rules! expect {
        ($usecase:ident, $infra_name:ident, $method_name:ident, $($in_exp:expr),*) => {
            concat_idents::concat_idents!(expect_method = expect_, $method_name {
                $usecase
                    .$infra_name
                    .expect_method()
                    .once()
                    .with(
                        $(
                            mockall::predicate::function(|input| {
                                assert_eq!(*input, $in_exp);
                                true
                            })
                        ),+
                    )
            })
        };
        ($usecase:ident, $infra_name:ident, $method_name:ident, ($($in_exp:expr),*) => $out_exp:expr) => {
            expect!($usecase, $infra_name, $method_name, $($in_exp),*)
                .return_const(Ok($out_exp))
        };
    }

    #[macro_export]
    macro_rules! expect_at_repository {
        ($usecase:ident, $method_name:ident, $($in_exp:expr),*) => {
            expect!($usecase, repository, $method_name, $($in_exp,)* MockConnection{})
        };
        ($usecase:ident, $method_name:ident, ($($in_exp:expr),*) => $out_exp:expr) => {
            expect!($usecase, repository, $method_name, ($($in_exp,)* MockConnection{}) => $out_exp)
        };
    }

    // #[macro_export]
    // macro_rules! expect {
    //     ($usecase:ident, $infra_name:ident, $method_name:ident, $out_exp:expr, $($in_exp:expr),*) => {
    //         concat_idents::concat_idents!(expect_method = expect_, $method_name {
    //             $usecase
    //                 .$infra_name
    //                 .expect_method()
    //                 .once()
    //                 .with(
    //                     $(
    //                         mockall::predicate::function(|input| {
    //                             assert_eq!(*input, $in_exp);
    //                             true
    //                         })
    //                     ),+
    //                 )
    //                 .return_const(Ok($out_exp))
    //         })
    //     };
    // }

    // #[macro_export]
    // macro_rules! expect_at_repository {
    //     ($usecase:ident, $method_name:ident, $out_exp:expr) => {
    //         expect!($usecase, repository, $method_name, $out_exp, MockConnection{})
    //     };
    //     ($usecase:ident, $method_name:ident, $out_exp:expr, $($in_exp:expr,)*) => {
    //         expect!($usecase, repository, $method_name, $out_exp, $($in_exp:expr,)* MockConnection{})
    //     };
    // }
}
