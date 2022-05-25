pub enum WeblabAttribute {
    Test,
    Library,
    Solution,
}

pub struct ProgrammingAssignment {
    pub title: &'static str,
    pub assignment_text: &'static str,

    pub library_visible: bool,
    pub spectest_stdout_visible: bool,
    pub test: &'static str,
    pub solution: &'static str,
    pub library: Option<&'static str>,
    pub test_template: &'static str,
    pub solution_template: &'static str,
    pub checklist: Option<Checklist>,
}

pub struct WeblabFolder {
    pub title: &'static str,
    pub assignment_text: &'static str,
    pub assignments: &'static [WeblabAssignment],
}

pub struct Checklist {
    pub items: Vec<&'static str>,
}

pub struct OpenQuestion {
    pub title: &'static str,

    pub assignment_text: &'static str,

    pub expected_answer: &'static str,
    pub checklist: Option<Checklist>,
}

pub struct InlineQuestionList {
    pub title: &'static str,

    pub assignment_text: &'static str,

    pub assignments: &'static [WeblabAssignment],
}

pub struct MCOption {
    pub text: &'static str,
    pub is_correct: bool,
}

#[derive(Copy, Clone)]
pub enum MCStyle {
    AllThatApply,
    NumCorrect(usize),
}

impl Default for MCStyle {
    fn default() -> Self {
        Self::NumCorrect(1)
    }
}

pub struct MCQuestion {
    pub title: &'static str,
    pub assignment_text: &'static str,

    pub options: &'static [MCOption],
    pub randomize: bool,
    pub style: MCStyle,
}

pub enum WeblabAssignment {
    Programming(ProgrammingAssignment),
    Open(OpenQuestion),
    MultipleChoice(MCQuestion),
    Folder(WeblabFolder),
    InlineQuestionList(InlineQuestionList),
}

impl WeblabAssignment {
    pub fn title(&self) -> &str {
        match self {
            WeblabAssignment::Programming(ProgrammingAssignment { title, .. }) => title,
            WeblabAssignment::Open(OpenQuestion { title, .. }) => title,
            WeblabAssignment::MultipleChoice(MCQuestion { title, .. }) => title,
            WeblabAssignment::Folder(WeblabFolder { title, .. }) => title,
            WeblabAssignment::InlineQuestionList(InlineQuestionList { title, .. }) => title,
        }
    }
    pub fn assignment_text(&self) -> &str {
        match self {
            WeblabAssignment::Programming(ProgrammingAssignment {
                assignment_text, ..
            }) => assignment_text,
            WeblabAssignment::Open(OpenQuestion {
                assignment_text, ..
            }) => assignment_text,
            WeblabAssignment::MultipleChoice(MCQuestion {
                assignment_text, ..
            }) => assignment_text,
            WeblabAssignment::Folder(WeblabFolder {
                assignment_text, ..
            }) => assignment_text,
            WeblabAssignment::InlineQuestionList(InlineQuestionList {
                assignment_text, ..
            }) => assignment_text,
        }
    }
}
