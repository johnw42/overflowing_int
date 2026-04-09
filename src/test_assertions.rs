#[macro_export]
macro_rules! prop_assert_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let left = $left;
        let right = $right;

        if left != right {
            return TestResult::error(format!(
                "prop failed at {}:{}:{}: left == right\nleft: `{:?}`\nright: `{:?}`{}",
                file!(), line!(), column!(),
                left, right))
        }
        TestResult::passed()
    }};
    ($left:expr, $right:expr, $($rest:tt)*) => {{
        let left = $left;
        let right = $right;

        if left != right {
            return TestResult::error(format!(
                "prop failed at {}:{}:{}: {}\nleft: `{:?}`\nright: `{:?}`{}",
                file!(), line!(), column!(),
                format_args!($($rest)*),
                left, right))
        }
        TestResult::passed()
    }};
}

#[macro_export]
macro_rules! prop_assert_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let left = $left;
        let right = $right;

        if left != right {
            return TestResult::error(format!(
                "prop failed at {}:{}:{}: left == right\nleft: `{:?}`\nright: `{:?}`{}",
                file!(), line!(), column!(),
                left, right))
        }
        TestResult::passed()
    }};
    ($left:expr, $right:expr, $($rest:tt)*) => {{
        let left = $left;
        let right = $right;

        if left != right {
            return TestResult::error(format!(
                "prop failed at {}:{}:{}: {}\nleft: `{:?}`\nright: `{:?}`{}",
                file!(), line!(), column!(),
                format_args!($($rest)*),
                left, right))
        }
        TestResult::passed()
    }};
}

#[macro_export]
macro_rules! prop_assert_ne {
    ($left:expr, $right:expr $(,)?) => {{
        let left = $left;
        let right = $right;

        if left == right {
            return TestResult::error(format!(
                "prop failed at {}:{}:{}: left != right\nleft: `{:?}`\nright: `{:?}`{}",
                file!(), line!(), column!(),
                left, right))
        }
        TestResult::passed()
    }};
    ($left:expr, $right:expr, $($rest:tt)*) => {{
        let left = $left;
        let right = $right;

        if left == right {
            return TestResult::error(format!(
                "prop failed at {}:{}:{}: {}\nleft: `{:?}`\nright: `{:?}`{}",
                file!(), line!(), column!(),
                format_args!($($rest)*),
                left, right))
        }
        TestResult::passed()
    }};
}
