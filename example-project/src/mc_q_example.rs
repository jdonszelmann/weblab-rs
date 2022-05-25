use weblab::mc_question;

mc_question! {
    title: "example mc",
    question:
    /// Explain how you answered this question
    ///
    /// # Markdown works here too
    explanation:
    /// This explains the correct answer. It's not and a bit!

    option "not" correct,
    option "a bit" correct,
    option "lots",

    expect 1 answers,
    randomize
}
