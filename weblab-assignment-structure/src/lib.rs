pub enum WeblabAttribute {
    Test,
    Library,
    Solution,
}

pub struct ProgrammingAssignment {
    pub title: &'static str,
    pub description: &'static str,

    pub library_visible: bool,
    pub spectest_stdout_visible: bool,
    pub test: &'static str,
    pub solution: &'static str,
    pub library: Option<&'static str>,
    pub test_template: &'static str,
    pub solution_template: &'static str,
    pub checklist: Option<Checklist>,
}

pub struct Checklist {
    pub items: Vec<&'static str>,
}

pub struct OpenQuestion {
    pub question: &'static str,

    pub expected_answer: &'static str,
    pub checklist: Option<Checklist>,
}

pub struct MCQuestion {
    pub options: Vec<&'static str>,
    pub answer: usize,
}

pub enum WeblabAssignment {
    Programming(ProgrammingAssignment),
    Open(OpenQuestion),
    MultipleChoice(MCQuestion),
    Folder(WeblabFolder),
}

pub struct WeblabFolder {
    pub title: &'static str,
    pub assignments: &'static [WeblabAssignment],
}
