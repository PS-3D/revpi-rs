// Basically does same as anyhow::ensure
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if $cond {
            return Err($err);
        }
    };
}

pub(crate) use ensure;
