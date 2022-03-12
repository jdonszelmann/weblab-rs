
# Weblab rs

Generate Weblab assignments from rust code. Works as a rust library.

## Installation

Just add the following line to your `Cargo.toml`
```bash
weblab = "<replace with current version>"
```

## Project structure

With weblab-rs you can make 3 kinds of assignments. Open-answer, multiple choice and programming exercises. 
The last needs the most explanation. A programming question is a *single file* with the following structure:

```rust
use weblab::weblab;

#[weblab(programming_assignment)]
/// This is an example assignment. This text will become the assignment text.
/// # You can use markdown here. Most editors (like clion) even shows it.
/// The markdown will also show on weblab.
///
/// The assignment here is: return the word "main"!
#[weblab(title = "test")] // otherwise the module name is used
mod assignment {
    #[weblab(solution)]
    mod solution {
        // In this module, the solution of your assignment goes.
        // Parts of the solution may be marked with template_only!{}
        // or solution_only!{}. If this is the case, those parts
        // will be added or removed only for the template version.
        // Like that you can reuse part of the solution as the template
        // of the assignment

        // It is recommended to put a function here in which the student's
        // code goes. However, a solution may also contain only types, or
        // any other rust structure you like.

        // note that you may get unused import warnings on template_only.
        // template_only is removed from the source code by the ecnlosing
        // attribute macros (that's how it works). This means that technically
        // it's never used. You don't even need to import it! However, for
        // syntax highlighting it may be useful to import it anyway.
        #[allow(unused_imports)]
        use weblab::{solution_only, template_only};

        pub fn main() -> &'static str {
            solution_only!{
                "main"
            }
            template_only!{
                todo!()
            }
        }
    }

    #[weblab(test)]
    mod test {
        // in this section, tests can be put.
        // imports (any import for that matter, also from the library)
        // need to be prefixed with `super`. This works both for the
        // offline project, and in the generated weblab assignment.
        // NOTE: crate-relative imports may work offline, but will *not*
        // work when the assignment is generated on weblab, since each
        // assignment will be put in its own, sepsarate module structure.
        // In tests, `template_only!{}` and `solution_only!{}` also work.

        use super::solution;

        // Tests are marked with the standard rust test annotation.
        // To try out the spec tests locally, simply run `cargo test`.
        // The code will be automatically configured such that the
        // solution will be equal to the reference solution, and the tests
        // will verify the reference solution.
        #[test]
        fn test() {
            assert_eq!(solution::main(), "main");
        }
    }

    // The library is optional.
    #[weblab(library)]
    mod library {}

    // optionally, if the solution template and reference solution are
    // very different, you may choose to add another module:
    // ```
    // #[weblab(solution_template)]
    // mod solution_template {}
    // ```
    //
    // You can do the same for tests:
    //
    // ```
    // #[weblab(test_template)]
    // mod test_template {}
    // ```
}


```

To generate weblab assignments (in uploadable zip form) from this structure
from this, make the root of the project a `main.rs` file similar to this:

```rust
#[weblab(main)]
fn main() { 
    // the main function will be automatically populated
    // with a nice CLI. Code you put in here will be ignored.  
}
```

This adds a command line interface to your project, which you
can now use to interact with the generation code. The following
commands are some of what's available:
```bash
cargo run -- generate folder
cargo run -- generate zip
cargo run -- generate zip --name='some_assignment'
cargo run -- generate folder --name='some_assigment'
cargo run -- generate folder --module='some::path::to::an::assignment'

# for all commands
cargo run -- --help 

# always available, simply runs the spectests
cargo test
```