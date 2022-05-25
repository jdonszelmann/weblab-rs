use weblab::{weblab_folder, weblab_main};

mod a;
mod open_q_example;
mod c;
mod mc_q_example;
mod question_list;

weblab_folder! {
    "test",

    a,
    open_q_example,
    mc_q_example,
    c,
    question_list,
}

fn main() {
    weblab_main!(self);
}
