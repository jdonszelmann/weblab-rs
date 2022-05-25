use clap::{Parser, Subcommand};
use regex::Captures;
use sanitize_filename::sanitize;
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io, process};
use path_slash::PathBufExt;
use walkdir::WalkDir;
use weblab_assignment_structure::{InlineQuestionList, MCOption, MCQuestion, MCStyle, OpenQuestion, ProgrammingAssignment, WeblabAssignment, WeblabFolder};
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Serialize)]
pub struct Alternative {
    #[serde(rename = "text")]
    pub text: &'static str,
    #[serde(rename = "correct")]
    pub correct: bool,
}

#[derive(Serialize, Default)]
struct Question {
    #[serde(rename = "type")]
    r#type: String,

    #[serde(rename = "language")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    language: Option<String>,

    #[serde(rename = "libraryVisible")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    library_visible: Option<bool>,
    #[serde(rename = "programOutputVisible")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    program_output_visible: Option<bool>,
    #[serde(rename = "specTestDetailsVisible")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    spec_test_details_visible: Option<bool>,

    #[serde(rename = "dockerImage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    docker_image: Option<String>,

    #[serde(rename = "editorLanguageId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    editor_language_id: Option<String>,
    #[serde(rename = "editorTestLanguageId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    editor_test_language_id: Option<String>,
    #[serde(rename = "editorLibraryLanguageId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    editor_library_language_id: Option<String>,

    #[serde(rename = "solutionFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    solution_file: Option<String>,
    #[serde(rename = "solutionTemplateFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    solution_template_file: Option<String>,
    #[serde(rename = "specTestFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    spec_test_file: Option<String>,
    #[serde(rename = "userTestTemplateFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    user_test_template_file: Option<String>,
    #[serde(rename = "libraryFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    library_file: Option<String>,
    #[serde(rename = "essayAnswerFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    essay_answer_file: Option<String>,

    #[serde(rename = "questionTextFile")]
    question_text_file: String,
    #[serde(rename = "title")]
    title: String,

    #[serde(rename = "explanation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    explanation: Option<String>,

    #[serde(rename = "randomOrder")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    random_order: Option<bool>,

    #[serde(rename = "selectN")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    num_answers_expected: Option<usize>,

    #[serde(rename = "alternatives")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    alternatives: Vec<Alternative>,
}


#[derive(Serialize)]
struct Child {
    #[serde(rename = "childAssignmentRelPath")]
    child_assignment_rel_path: String,
}

#[derive(Serialize)]
struct Folder {
    #[serde(rename = "title")]
    title: String,

    #[serde(rename = "children")]
    children: Vec<Child>,

    #[serde(rename = "displayInline")]
    display_inline: bool,

    #[serde(rename = "descriptionFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    description_file: Option<String>,
}

#[derive(Serialize)]
struct AssignmentData {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "question")]
    question: Option<Question>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    folder: Option<Folder>,
}

impl AssignmentData {
    pub fn new_folder(title: &str, relative_paths: &[&str], inline: bool, assignment_text: &str) -> Self {
        Self {
            question: None,
            folder: Some(Folder {
                title: title.to_string(),
                children: relative_paths
                    .iter()
                    .map(|i| Child {
                        child_assignment_rel_path: i.to_string(),
                    })
                    .collect(),
                display_inline: inline,
                description_file: (!assignment_text.is_empty()).then(|| {
                    "assignment_description.md".to_string()
                }),
            }),
        }
    }

    pub fn new_programming(
        title: &str,
        library_visible: bool,
        program_output_visible: bool,
        spec_test_details_visible: bool,
    ) -> Self {
        Self {
            question: Some(Question {
                r#type: "ProgrammingQuestion".to_string(),
                language: Some("docker:cese4000-2022-rust".to_string()),
                library_visible: Some(library_visible),
                program_output_visible: Some(program_output_visible),
                spec_test_details_visible: Some(spec_test_details_visible),
                docker_image: Some("".to_string()),

                editor_language_id: Some("rust".to_string()),
                editor_test_language_id: Some("rust".to_string()),
                editor_library_language_id: Some("rust".to_string()),

                solution_file: Some("solution.rs".to_string()),
                solution_template_file: Some("solution_template.rs".to_string()),
                spec_test_file: Some("test.rs".to_string()),
                user_test_template_file: Some("test_template.rs".to_string()),
                library_file: Some("library.rs".to_string()),
                question_text_file: "question.md".to_string(),

                title: title.to_string(),

                ..Default::default()
            }),
            folder: None,
        }
    }

    pub fn new_mc(
        title: &str,
        explanation: &str,
        randomize: bool,
        style: MCStyle,
        options: &[MCOption],
    ) -> Self {
        Self {
            question: Some(Question {
                r#type: "MultipleChoiceQuestion".to_string(),
                question_text_file: "question.md".to_string(),
                explanation: Some(explanation.to_string()),

                title: title.to_string(),
                random_order: Some(randomize),

                num_answers_expected: Some(match style {
                    MCStyle::AllThatApply => 0,
                    MCStyle::NumCorrect(i) => i,
                }),

                alternatives: options
                    .iter()
                    .map(|i| Alternative {
                        text: i.text,
                        correct: i.is_correct
                    }).collect(),

                ..Default::default()
            }),
            folder: None,
        }
    }

    pub fn new_open(
        title: &str,
    ) -> Self {
        Self {
            question: Some(Question {
                r#type: "EssayQuestion".to_string(),
                question_text_file: "question.md".to_string(),
                essay_answer_file: Some("answer.md".to_string()),

                title: title.to_string(),

                ..Default::default()
            }),
            folder: None,
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Generate {
        #[clap(subcommand)]
        filetype: GenerateType,
    },
}

#[derive(Clone, PartialEq, Eq, Subcommand)]
enum GenerateType {
    Zip {
        #[clap(default_value_t = String::from("output.zip"))]
        output: String,
    },
    Folder {
        #[clap(default_value_t = String::from("output"))]
        output: String,
    },
}

#[derive(Debug)]
struct StringError(String);

impl Error for StringError {}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn generate_zip(
    path: impl AsRef<Path>,
    assignment: WeblabAssignment,
) -> Result<(), Box<dyn Error>> {
    let mut folder_path = path.as_ref().to_path_buf();
    folder_path.set_extension("");
    generate_folder(&folder_path, assignment)?;
    let f = File::create(path)?;
    let mut w = ZipWriter::new(f);

    for entry in WalkDir::new(&folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let metadata = entry.metadata()?;

        let relative = pathdiff::diff_paths(entry.path(), &folder_path).expect("must be prefix");

        if metadata.is_dir() {
            w.add_directory(relative.to_slash_lossy(), FileOptions::default())?;
        } else {
            w.start_file(relative.to_slash_lossy(), FileOptions::default())?;
            let mut f = File::open(entry.path())?;
            let mut contents = Vec::new();
            f.read_to_end(&mut contents)?;
            w.write_all(&contents)?;
        }
    }

    w.finish()?;

    Ok(())
}

fn write_and_fmt<P: AsRef<Path>, S: ToString>(path: P, code: S) -> io::Result<()> {
    let r = regex::Regex::new(r#"#\[doc *= *"(.*)"\]"#).expect("should compile");
    let rq = regex::Regex::new(r#"\\(.)"#).expect("should compile");

    fs::write(&path, code.to_string())?;

    process::Command::new("rustfmt")
        .arg(path.as_ref())
        .spawn()?
        .wait()?;

    let code = fs::read_to_string(&path)?;
    let replaced = r.replace_all(&code, |caps: &Captures| {
        format!("///{}", rq.replace_all(&caps[1], "$1"))
    });

    fs::write(&path, replaced.as_ref())?;

    Ok(())
}

fn generate_folder_internal(
    path: impl AsRef<Path>,
    WeblabFolder { title, assignment_text, assignments }: &WeblabFolder,
    inline: bool,
) -> Result<(), Box<dyn Error>> {
    let p = path.as_ref().to_path_buf().join(sanitize(title));

    std::fs::create_dir_all(&p)?;

    let mut f = File::create(p.join("assignment-data.json"))?;
    let s = serde_json::to_string_pretty(&AssignmentData::new_folder(
        title,
        &assignments.iter().map(|i| i.title()).collect::<Vec<_>>(),
        inline,
        assignment_text,
    ))?;
    f.write_all(s.as_bytes())?;

    if !assignment_text.is_empty() {
        let mut description = File::create(p.join("assignment_description.md"))?;
        description.write_all(assignment_text.as_bytes())?;
    }

    for i in *assignments {
        recursive_generate_folder(&p, i)?
    }

    Ok(())
}


fn recursive_generate_folder(
    path: impl AsRef<Path>,
    assignment: &WeblabAssignment,
) -> Result<(), Box<dyn Error>> {
    match assignment {
        WeblabAssignment::Programming(ProgrammingAssignment {
                                          title,
                                          assignment_text,
                                          mut library_visible,
                                          spectest_stdout_visible: _,
                                          test,
                                          solution,
                                          library,
                                          test_template,
                                          solution_template,
                                          checklist: _,
                                      }) => {
            let p = &(*path.as_ref()).to_path_buf().join(&sanitize(title));
            std::fs::create_dir_all(&p)?;

            if let Some(l) = library {
                write_and_fmt(p.join("library.rs"), l)?;
            } else {
                // write but force invisible when there's no library
                write_and_fmt(p.join("library.rs"), "")?;
                library_visible = false;
            }

            write_and_fmt(p.join("solution.rs"), solution)?;
            write_and_fmt(p.join("solution_template.rs"), solution_template)?;
            write_and_fmt(p.join("test.rs"), test)?;
            write_and_fmt(p.join("test_template.rs"), test_template)?;

            let mut f = File::create(p.join("question.md"))?;
            f.write_all(assignment_text.as_bytes())?;

            let mut f = File::create(p.join("assignment-data.json"))?;
            let s = serde_json::to_string_pretty(&AssignmentData::new_programming(
                title,
                library_visible,
                false,
                false,
            ))?;
            f.write_all(s.as_bytes())?;
        }
        WeblabAssignment::Open(OpenQuestion { title, assignment_text, expected_answer, .. }) => {
            let p = &(*path.as_ref()).to_path_buf().join(&sanitize(title));
            std::fs::create_dir_all(&p)?;

            let mut f = File::create(p.join("answer.md"))?;
            f.write_all(expected_answer.as_bytes())?;

            let mut f = File::create(p.join("question.md"))?;
            f.write_all(assignment_text.as_bytes())?;


            let mut f = File::create(p.join("assignment-data.json"))?;
            let s = serde_json::to_string_pretty(&AssignmentData::new_open(
                title,
            ))?;
            f.write_all(s.as_bytes())?;
        }
        WeblabAssignment::MultipleChoice(MCQuestion { title, assignment_text, options, randomize, style }) => {
            let p = &(*path.as_ref()).to_path_buf().join(&sanitize(title));
            std::fs::create_dir_all(&p)?;

            let mut f = File::create(p.join("question.md"))?;
            f.write_all(assignment_text.as_bytes())?;


            let mut f = File::create(p.join("assignment-data.json"))?;
            let s = serde_json::to_string_pretty(&AssignmentData::new_mc(
                title,
                "",
                *randomize,
                *style,
                *options,
            ))?;
            f.write_all(s.as_bytes())?;
        }
        WeblabAssignment::Folder(wf) => {
            generate_folder_internal(path, wf, false)?;
        }
        WeblabAssignment::InlineQuestionList(InlineQuestionList { title, assignment_text, assignments }) => {
            generate_folder_internal(path, &WeblabFolder {
                title,
                assignment_text,
                assignments,
            }, true)?;
        }
    }

    Ok(())
}

fn generate_folder(
    path: impl AsRef<Path>,
    assignment: WeblabAssignment,
) -> Result<(), Box<dyn Error>> {
    if path.as_ref().exists() {
        std::fs::remove_dir_all(&path)?;
    }
    std::fs::create_dir_all(&path)?;

    recursive_generate_folder(path, &assignment)?;

    Ok(())
}

fn check_folder(title: &str, assignments: &[WeblabAssignment]) -> Result<(), Box<dyn Error>> {
    let mut titles = HashSet::new();

    for i in assignments {
        if titles.contains(i.title()) {
            return Err(Box::new(StringError(format!("folder `{title}` contains multiple assignments with the name `{}`, which leads to ambiguity", i.title()))));
        } else {
            titles.insert(i.title());
            check_assignment_tree(i)?;
        }
    }

    Ok(())
}

fn check_assignment_tree(assignment: &WeblabAssignment) -> Result<(), Box<dyn Error>> {
    match assignment {
        WeblabAssignment::Programming(_) => {}
        WeblabAssignment::Open(_) => {}
        WeblabAssignment::MultipleChoice(_) => {}
        WeblabAssignment::Folder(WeblabFolder {
                                     title,
                                     assignments,
                                     assignment_text: _,
                                 }) => {
            check_folder(title, assignments)?;
        }
        WeblabAssignment::InlineQuestionList(InlineQuestionList { title, assignment_text: _, assignments }) => {
            check_folder(title, assignments)?;
        }
    }

    Ok(())
}

pub fn error_main(assignment: WeblabAssignment, args: &[String]) -> Result<(), Box<dyn Error>> {
    check_assignment_tree(&assignment)?;

    let args: Cli = Cli::parse_from(args);

    match args.command {
        Command::Generate { filetype } => match filetype {
            GenerateType::Zip { output, .. } => {
                let path = PathBuf::from(output);
                if let Some("zip") = path
                    .extension()
                    .map(|i| i.to_string_lossy().to_string())
                    .as_deref()
                {
                    generate_zip(path, assignment)?;
                } else {
                    return Err(Box::new(StringError(
                        "expected output file to have a zip extension".to_string(),
                    )));
                }
            }
            GenerateType::Folder { output, .. } => {
                let path = PathBuf::from(output);
                generate_folder(path, assignment)?
            }
        },
    }

    Ok(())
}

pub fn main(assignment: WeblabAssignment, args: &[String]) {
    if let Err(e) = error_main(assignment, args) {
        eprintln!("{}", e);
    }
}
