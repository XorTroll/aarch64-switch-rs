#![macro_use]

#[macro_export]
macro_rules! bit {
    ($val:expr) => {
        (1 << $val)
    };
}

#[macro_export]
macro_rules! util_return_if {
    ($cond:expr, $ret:expr) => {
        if $cond {
            return $ret;
        }
    }
}

#[macro_export]
macro_rules! util_return_unless {
    ($cond:expr, $ret:expr) => {
        if !$cond {
            return $ret;
        }
    }
}

#[macro_export]
macro_rules! write_bits {
    ($start:expr, $end:expr, $value:expr, $data:expr) => {
        $value = ($value & (!( ((1 << ($end - $start + 1)) - 1) << $start ))) | ($data << $start);
    };
}

#[macro_export]
macro_rules! read_bits {
    ($start:expr, $end:expr, $value:expr) => {
        ($value & (((1 << ($end - $start + 1)) - 1) << $start)) >> $start
    };
}