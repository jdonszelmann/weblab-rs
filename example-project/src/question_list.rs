use weblab::inline_question_list;

inline_question_list! {
    title: "example of inline question list",
    question:
    /// With a question list, you can make multiple inline questions on a single page.


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


    open_question! {
        title: "example open",
        question:
        /// Explain how you answered this question
        ///
        /// # Markdown works here too

        answer:
        /// while typing
    }
}
