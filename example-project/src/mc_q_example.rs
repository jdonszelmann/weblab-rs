use weblab::mc_question;

mc_question! {
    title: "example mc",
    question:
    /// Explain how you answered this question
    ///
    /// # Markdown works here too

    option "not" correct,
    option "a bit" correct,
    option "lots",

    expect 1 answers,
    randomize
}
