use {
    mollusk_svm::{program::loader_keys::LOADER_V3, Mollusk},
    solana_instruction::Instruction,
    solana_pubkey::Pubkey,
    solana_svm_log_collector::LogCollector,
    std::{
        fs,
        panic::{self, AssertUnwindSafe},
        path::{Path, PathBuf},
        process::ExitCode,
    },
};

const REFERENCE: &str = "sbf";
const CANDIDATE: &str = "upstream";

#[derive(PartialEq)]
struct Run {
    result: String,
    logs: Vec<String>,
}

enum Outcome {
    Match { cases: usize },
    Mismatch(Vec<String>),
    Skipped(Vec<PathBuf>),
}

fn words(words: [i64; 3]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

fn inputs(program: &str) -> Vec<Vec<u8>> {
    match program {
        "hello_arith" => vec![
            words([-7, 2, 0x1234_5680]),
            words([7, -2, -1]),
            words([-7, -2, 0x7f]),
            words([0x7fff_ffff, 1, 0x0102_0304]),
            words([i64::MIN, -1, 0]),
            words([1, 0, 0]),
        ],
        _ => vec![Vec::new()],
    }
}

fn run(elf: &[u8], data: &[u8]) -> Run {
    let program_id = Pubkey::new_from_array([7; 32]);
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut mollusk = Mollusk::default();
        mollusk.add_program_with_loader_and_elf(&program_id, &LOADER_V3, elf);
        let logger = LogCollector::new_ref();
        mollusk.logger = Some(logger.clone());
        let instruction = Instruction::new_with_bytes(program_id, data, vec![]);
        let result = mollusk.process_instruction(&instruction, &[]);
        let logs = logger
            .borrow()
            .get_recorded_content()
            .iter()
            .filter(|line| line.starts_with("Program log:"))
            .cloned()
            .collect();
        Run {
            result: format!("{:?}", result.program_result),
            logs,
        }
    }));
    outcome.unwrap_or_else(|panic| Run {
        result: format!(
            "mollusk panicked: {}",
            panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("(no message)")
        ),
        logs: Vec::new(),
    })
}

fn describe(toolchain: &str, run: &Run) -> Vec<String> {
    let mut lines = vec![format!("  {toolchain:<9} {}", run.result)];
    lines.extend(run.logs.iter().map(|log| format!("  {:<9} {log}", "")));
    lines
}

fn compare(out: &Path, program: &str) -> Outcome {
    let paths = [REFERENCE, CANDIDATE].map(|dir| out.join(dir).join(format!("{program}.so")));
    let missing: Vec<PathBuf> = paths.iter().filter(|p| !p.exists()).cloned().collect();
    if !missing.is_empty() {
        return Outcome::Skipped(missing);
    }
    let [reference, candidate] = paths.map(|p| fs::read(p).expect("read program"));

    let cases = inputs(program);
    let mut diffs = Vec::new();
    for (case, data) in cases.iter().enumerate() {
        let (expected, actual) = (run(&reference, data), run(&candidate, data));
        if expected != actual {
            diffs.push(format!("  input #{case}: {data:02x?}"));
            diffs.extend(describe(REFERENCE, &expected));
            diffs.extend(describe(CANDIDATE, &actual));
        }
    }
    if diffs.is_empty() {
        Outcome::Match { cases: cases.len() }
    } else {
        Outcome::Mismatch(diffs)
    }
}

fn main() -> ExitCode {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "error");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut programs: Vec<String> = fs::read_dir(root.join("programs"))
        .expect("read programs/")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().replace('-', "_"))
        .collect();
    programs.sort();

    let out = root.join("out");
    let (mut mismatched, mut skipped) = (0, Vec::new());
    for program in &programs {
        match compare(&out, program) {
            Outcome::Match { cases } => {
                println!("{program:<16} ok        {cases} input(s) match");
            }
            Outcome::Mismatch(diffs) => {
                mismatched += 1;
                println!("{program:<16} MISMATCH");
                diffs.iter().for_each(|line| println!("{line}"));
            }
            Outcome::Skipped(missing) => {
                println!("{program:<16} SKIPPED   not built:");
                for path in &missing {
                    let shown = path.strip_prefix(&root).unwrap_or(path);
                    println!("  {}", shown.display());
                }
                skipped.push(program.as_str());
            }
        }
    }

    if !skipped.is_empty() {
        let bar = "!".repeat(72);
        println!("\n{bar}");
        println!(
            "!! {} of {} programs SKIPPED, not compared: {}",
            skipped.len(),
            programs.len(),
            skipped.join(", ")
        );
        println!("!! A missing .so usually means its build failed.");
        println!("{bar}");
    }
    if mismatched > 0 || !skipped.is_empty() {
        return ExitCode::FAILURE;
    }
    println!("\nall {} programs match", programs.len());
    ExitCode::SUCCESS
}
