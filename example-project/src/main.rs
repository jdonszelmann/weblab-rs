use weblab::{weblab_folder, weblab_main};

mod a;
mod b;
mod c;

weblab_folder! {
    "test",

    a,
    //b
    c,
}

fn main() {
    weblab_main!(self);
}
