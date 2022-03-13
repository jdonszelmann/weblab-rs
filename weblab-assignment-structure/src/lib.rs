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
    pub title: &'static str,

    pub question: &'static str,

    pub expected_answer: &'static str,
    pub checklist: Option<Checklist>,
}

pub struct MCQuestion {
    pub title: &'static str,

    pub options: Vec<&'static str>,
    pub answer: usize,
}

pub enum WeblabAssignment {
    Programming(ProgrammingAssignment),
    Open(OpenQuestion),
    MultipleChoice(MCQuestion),
    Folder(WeblabFolder),
}

impl WeblabAssignment {
    pub fn title(&self) -> &str {
        match self {
            WeblabAssignment::Programming(ProgrammingAssignment { title, .. }) => title,
            WeblabAssignment::Open(OpenQuestion { title, .. }) => title,
            WeblabAssignment::MultipleChoice(MCQuestion { title, .. }) => title,
            WeblabAssignment::Folder(WeblabFolder { title, .. }) => title,
        }
    }
}

pub struct WeblabFolder {
    pub title: &'static str,
    pub assignments: &'static [WeblabAssignment],
}
