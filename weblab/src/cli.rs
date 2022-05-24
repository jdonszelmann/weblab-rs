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
use weblab_assignment_structure::{ProgrammingAssignment, WeblabAssignment, WeblabFolder};
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Serialize)]
struct Question {
    #[serde(rename = "type")]
    r#type: String,

    #[serde(rename = "language")]
    language: String,

    #[serde(rename = "libraryVisible")]
    library_visible: bool,
    #[serde(rename = "programOutputVisible")]
    program_output_visible: bool,
    #[serde(rename = "specTestDetailsVisible")]
    spec_test_details_visible: bool,

    #[serde(rename = "dockerImage")]
    docker_image: String,

    #[serde(rename = "editorLanguageId")]
    editor_language_id: String,
    #[serde(rename = "editorTestLanguageId")]
    editor_test_language_id: String,
    #[serde(rename = "editorLibraryLanguageId")]
    editor_library_language_id: String,

    #[serde(rename = "solutionFile")]
    solution_file: String,
    #[serde(rename = "solutionTemplateFile")]
    solution_template_file: String,
    #[serde(rename = "specTestFile")]
    spec_test_file: String,
    #[serde(rename = "userTestTemplateFile")]
    user_test_template_file: String,
    #[serde(rename = "libraryFile")]
    library_file: String,

    #[serde(rename = "questionTextFile")]
    question_text_file: String,
    #[serde(rename = "title")]
    title: String,
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
    pub fn new_folder(title: &str, relative_paths: &[&str]) -> Self {
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
                language: "docker:cese4000-2022-rust".to_string(),
                library_visible,
                program_output_visible,
                spec_test_details_visible,
                docker_image: "".to_string(),

                editor_language_id: "rust".to_string(),
                editor_test_language_id: "rust".to_string(),
                editor_library_language_id: "rust".to_string(),

                solution_file: "solution.rs".to_string(),
                solution_template_file: "solution_template.rs".to_string(),
                spec_test_file: "test.rs".to_string(),
                user_test_template_file: "test_template.rs".to_string(),
                library_file: "library.rs".to_string(),
                question_text_file: "question.md".to_string(),

                title: title.to_string(),
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
        #[clap(default_value_t=String::from("output.zip"))]
        output: String,
    },
    Folder {
        #[clap(default_value_t=String::from("output"))]
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
        WeblabAssignment::Open(_) => {}
        WeblabAssignment::MultipleChoice(_) => {}
        WeblabAssignment::Folder(WeblabFolder {
            title,
            assignments,
            assignment_text: _,
        }) => {
            let p = path.as_ref().to_path_buf().join(sanitize(title));

            std::fs::create_dir_all(&p)?;

            let mut f = File::create(p.join("assignment-data.json"))?;
            let s = serde_json::to_string_pretty(&AssignmentData::new_folder(
                title,
                &assignments.iter().map(|i| i.title()).collect::<Vec<_>>(),
            ))?;
            f.write_all(s.as_bytes())?;

            for i in *assignments {
                recursive_generate_folder(&p, i)?
            }
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
            let mut titles = HashSet::new();

            for i in *assignments {
                if titles.contains(i.title()) {
                    return Err(Box::new(StringError(format!("folder `{title}` contains multiple assignments with the name `{}`, which leads to ambiguity", i.title()))));
                } else {
                    titles.insert(i.title());
                    check_assignment_tree(i)?;
                }
            }
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
