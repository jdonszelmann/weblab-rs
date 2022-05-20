use weblab::weblab;

#[weblab(programming_assignment)]
/// This is an example assignment.
/// # You can use markdown here. Most editors (like clion) even shows it.
/// The markdown will also show on weblab-macros.
#[weblab(title = "test_1")] // otherwise the module name is used
mod assignment {
    #[weblab(solution)]
    mod solution {
        /// Let's use a doc comment here
        pub fn main() {
            println!("main!");
            // todo!()
        }
    }

    #[weblab(test)]
    mod test {
        use super::solution;

        #[test]
        fn test() {
            solution::main();
        }
    }

    #[weblab(library)]
    mod library {}
}
