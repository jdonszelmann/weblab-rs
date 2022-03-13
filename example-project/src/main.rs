use weblab::{weblab_folder, weblab};

mod a;
mod b;
mod c;

weblab_folder! {
    "test",

    a,
    // b,
}

#[weblab(main)]
fn main() {}
