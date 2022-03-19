///! for library docs, refer to [https://github.com/jonay2000/weblab-rs](https://github.com/jonay2000/weblab-rs)
pub use weblab_assignment_structure::*;
pub use weblab_macros::*;

pub mod cli;


#[macro_export]
macro_rules! weblab_assignment_info {
    (self) => {
        {
            let w: $crate::WeblabAssignment = __WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO;
            w
        }
    };
    ($root_module: path) => {{
        use $root_module as weblab_module;
        let w: $crate::WeblabAssignment = weblab_module::__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO;
        w
    }};
}

#[macro_export]
macro_rules! weblab_main {
    (self, $($args: expr),*) => {
        $crate::cli::main(__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO, &[$($args.to_string()),*]);
    };
    (self) => {{
        $crate::cli::main(__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO, &std::env::args().collect::<Vec<String>>());
    }};
    ($root_module: path, $($args: expr),*) => {
        use $root_module as weblab_module;
        $crate::cli::main(weblab_module::__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO, &[$($args.to_string()),*]);
    };
    ($root_module: path) => {{
        use $root_module as weblab_module;
        $crate::cli::main(weblab_module::__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO, &std::env::args().collect::<Vec<String>>());
    }};
}

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
                assignment_text: "",
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
