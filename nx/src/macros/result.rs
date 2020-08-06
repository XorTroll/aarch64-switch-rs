#![macro_use]

#[macro_export]
macro_rules! result_define {
    ($name:ident: $value:expr) => {
        pub struct $name;
        
        impl $crate::result::ResultBase for $name {
            fn get_value() -> u32 {
                $value
            }
            
            fn get_module() -> u32 {
                unpack_module($value)
            }
            
            fn get_description() -> u32 {
                unpack_description($value)
            }
        }
    };
    ($name:ident: $module:expr, $description:expr) => {
        pub struct $name;
        
        impl $crate::result::ResultBase for $name {
            fn get_value() -> u32 {
                pack_value(Self::get_module(), Self::get_description())
            }
            
            fn get_module() -> u32 {
                $module
            }
            
            fn get_description() -> u32 {
                $description
            }
        }
    };
}

#[macro_export]
macro_rules! result_define_group {
    ($module:expr => { $( $name:ident: $description:expr ),* }) => {
        $(
            pub struct $name;
        
            impl $crate::result::ResultBase for $name {
                fn get_value() -> u32 {
                    pack_value(Self::get_module(), Self::get_description())
                }
                
                fn get_module() -> u32 {
                    $module
                }
                
                fn get_description() -> u32 {
                    $description
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! result_lib_define_group {
    ($submodule:expr => { $( $name:ident: $description:expr ),* }) => {
        $(
            pub struct $name;
        
            impl $crate::result::ResultBase for $name {
                fn get_value() -> u32 {
                    pack_value(Self::get_module(), Self::get_description())
                }
                
                fn get_module() -> u32 {
                    LIBRARY_MODULE
                }
                
                fn get_description() -> u32 {
                    $submodule * 100 + $description
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! result_return_if {
    ($cond:expr, $res:ty) => {
        if $cond {
            return Err($crate::result::ResultCode::from::<$res>());
        }
    };
    ($cond:expr, $res:literal) => {
        if $cond {
            return Err($crate::result::ResultCode::new($res));
        }
    }
}

#[macro_export]
macro_rules! result_return_unless {
    ($cond:expr, $res:ty) => {
        if !$cond {
            return Err($crate::result::ResultCode::from::<$res>());
        }
    };
    ($cond:expr, $res:literal) => {
        if !$cond {
            return Err($crate::result::ResultCode::new($res));
        }
    }
}

#[macro_export]
macro_rules! result_try {
    ($rc:expr) => {
        if $rc.is_failure() {
            return Err($rc);
        }
    };
}

