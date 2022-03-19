use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};
use simple_xml_serialize::XMLElement;
use simple_xml_serialize_macro::xml_element;

#[xml_element("testsuites")]
struct TestSuites {
    #[sxs_type_multi_element]
    testsuites: Vec<TestSuite>
}

#[xml_element("testsuite")]
struct TestSuite {
    #[sxs_type_multi_element]
    testcases: Vec<TestCase>
}

#[xml_element("testcase")]
struct TestCase {
    #[sxs_type_attr]
    name: String,
    #[sxs_type_element]
    failure: Option<Failure>
}

#[xml_element("failure")]
struct Failure {
    #[sxs_type_text]
    message: String,
}

#[derive(Deserialize)]
#[serde(tag="event")]
enum MessageType {
    #[serde(rename="suite")]
    Suite,
    #[serde(rename="test")]
    Test
}

#[derive(Deserialize)]
#[serde(tag="event")]
enum TestEventType {
    #[serde(rename="started")]
    Started {
        name: String
    },
    #[serde(rename="failed")]
    Failed {
        name: String
    },
    #[serde(rename="ok")]
    Ok {
        name: String
    }
}

#[derive(Deserialize)]
#[serde(tag="event")]
enum SuiteEventType {
    #[serde(rename="ok")]
    Ok {
        passed: usize,
        failed: usize,
        allowed_fail: usize,
        ignored: usize,
        measured: usize,
        filtered_out: usize,
    },
    #[serde(rename="started")]
    Started {
        test_count: usize,
    }
}

#[derive(Deserialize)]
#[serde(tag="type")]
enum TestReportMessage {
    #[serde(rename="test")]
    Test {
        #[serde(flatten)]
        event: TestEventType,

        #[serde(flatten)]
        other: serde_json::Value
    },

    #[serde(rename="suite")]
    Suite {
        #[serde(flatten)]
        event: SuiteEventType,

        #[serde(flatten)]
        other: serde_json::Value
    }
}

fn parse_test_output(stream: &[u8]) -> Vec<TestReportMessage> {
    let mut res = Vec::new();

    for i in stream.split(|i| i == &b'\n') {
        match serde_json::from_slice(i) {
            Ok(i) => res.push(i),
            Err(_) => {
                // eprintln!("{}\n{e}", String::from_utf8_lossy(i))
            }
        }
    }

    res
}

enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Unknown,
}

fn convert_to_junit(inp: Vec<TestReportMessage>) -> TestSuites {
    let mut tests = HashMap::new();

    for msg in inp {
        match msg {
            TestReportMessage::Test {event, other: _} => match event {
                TestEventType::Started { name } => {
                    tests.insert(name, TestStatus::Unknown);
                }
                TestEventType::Ok { name } => {
                    tests.insert(name, TestStatus::Passed);
                }
                TestEventType::Failed { name } => {
                    tests.insert(name, TestStatus::Failed);
                }
            }
            TestReportMessage::Suite { .. } => {}
        }
    }

    let mut testcases = Vec::new();

    for (name, t) in tests {
        testcases.push(TestCase {
            name,
            failure: match t {
                TestStatus::Passed => {
                    None
                }
                TestStatus::Failed => {
                    Some(Failure {
                        message: "failed".to_string()
                    })
                }
                TestStatus::Skipped => {
                    None
                }
                TestStatus::Unknown => {
                    None
                }
            }
        });
    }

    TestSuites {
        testsuites: vec![TestSuite {
            testcases,
        }]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("running tests...");
    let cmd = Command::new("cargo")
        .arg("test")
        .arg("--offline")
        .arg("--color=always")
        .arg("--no-fail-fast")
        .arg("--")
        .arg("--format=json")
        .arg("--test-threads").arg("3")
        .arg("--nocapture")
        .arg("-Z").arg("unstable-options")
        .output()?;

    println!("writing stdout and stderr");

    let path = PathBuf::from(std::env::args().nth(1).expect("expected path to put output files"));
    std::fs::remove_dir(&path);
    std::env::set_current_dir(path);

    let stdout = cmd.stdout;
    let stderr = cmd.stderr;

    File::create("stdout.txt")?.write_all(&stdout)?;
    File::create("stderr.txt")?.write_all(&stderr)?;

    println!("parsing test data");

    let messages = parse_test_output(&stdout);

    let junit = convert_to_junit(messages);
    let xml = XMLElement::from(junit);

    println!("writing xml");

    let res = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml.to_string_pretty("\n", "  "));
    File::create("results.xml")?.write_all(res.as_bytes())?;

    println!("done!");

    Ok(())
}