///! for library docs, refer to [https://github.com/jonay2000/weblab-rs](https://github.com/jonay2000/weblab-rs)
pub use weblab_assignment_structure::*;
pub use weblab_macros::*;

#[macro_export]
macro_rules! solution_only {
    ($($tt: tt)*) => {
        $($tt)*
    };
}

#[macro_export]
macro_rules! template_only {
    ($($tt: tt)*) => {
        $($tt)*
    };
}

#[macro_export]
macro_rules! weblab_folder {
    ($title: literal, $($module: path),* $(,)?) => {
        pub mod __WEBLAB_ASSIGNMENT_METADATA {
            use $crate::*;

            pub const ASSIGNMENT_INFO: WeblabAssignment = WeblabAssignment::Folder(WeblabFolder {
                title: $title,
                assignments: &[
                    $(
                        {
                            use super::*;
                            use $module as weblab_module;

                            weblab_module::__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO
                        }
                    ),*
                ],
            });
        }
    };
    ($name: ident, $($module: path),* $(,)?) => {
        weblab_folder!(stringify!($name), $($module),*);
    };
}

#[macro_export]
macro_rules! main {
    () => {
        fn main() {}
    };
}
