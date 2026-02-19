use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
    loop {
        if print_prompt().is_err() {
            eprintln!("failed to write prompt");
            continue;
        }

        let input = match read_input() {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(e) => {
                eprintln!("input error: {}", e);
                continue;
            }
        };

        if input == "exit" {
            break;
        }

        if let Err(e) = run_command_line(&input) {
            eprintln!("{}", e);
        }
    }
}

fn print_prompt() -> io::Result<()> {
    print!("mysh> ");
    io::stdout().flush()
}

fn read_input() -> io::Result<Option<String>> {
    let mut input = String::new();
    let bytes = io::stdin().read_line(&mut input)?;
    if bytes == 0 {
        return Ok(None);
    }
    let input = input.trim().to_string();
    if input.is_empty() {
        return Ok(Some(String::new()));
    }
    Ok(Some(input))
}

fn run_command_line(input: &str) -> Result<(), String> {
    let commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    let mut previous_stdout = None;

    for (i, cmd) in commands.iter().enumerate() {
        let mut parts = cmd.split_whitespace();
        let program = parts.next().ok_or("empty command")?;
        let args: Vec<&str> = parts.collect();

        if program == "cd" {
            let dir = args.get(0).copied().unwrap_or("/");
            env::set_current_dir(dir)
                .map_err(|e| format!("cd: {}", e))?;
            return Ok(());
        }

        let mut command = Command::new(program);
        command.args(&args);

        if let Some(stdout) = previous_stdout {
            command.stdin(Stdio::from(stdout));
        }

        if i < commands.len() - 1 {
            command.stdout(Stdio::piped());
        }

        let mut child = command
            .spawn()
            .map_err(|_| format!("command not found: {}", program))?;

        previous_stdout = child.stdout.take();

        child.wait().map_err(|e| e.to_string())?;
    }

    Ok(())
}
