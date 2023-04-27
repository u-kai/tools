use std::{
    io::{stdin, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

fn main() {
    loop {
        print_prompt();
        let mut line = String::new();
        stdin().read_line(&mut line).expect("Failed to read line");
        parse_prompt_line(line);
    }
}

#[cfg(target_os = "windows")]
const USER_ENV_KEY: &'static str = "USERNAME";
#[cfg(not(target_os = "windows"))]
const USER_ENV_KEY: &'static str = "USER";
fn sys_user() -> String {
    match std::env::var(USER_ENV_KEY) {
        Ok(s) => s,
        Err(_e) => "user".to_string(),
    }
}

fn print_prompt() {
    print!("{} dsh >> ", sys_user());
    std::io::stdout().flush().unwrap();
}

fn parse_prompt_line(line: String) {
    match run_command(line) {
        Ok(()) => (),
        Err(e) => println!("{:#?}", e),
    }
}

fn run_command(line: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut commands = line.trim_end().split(" ");
    let Some(command) = &commands.next() else {
        return Ok(())
    };
    let mut cmd = Command::new(command);
    cmd.args(commands);

    match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(child) => {
            if let Some(stdout) = child.stdout {
                let stdout_reader = BufReader::new(stdout);
                for line in stdout_reader.lines() {
                    println!("{}", line?);
                }
            };
            if let Some(stderr) = child.stderr {
                let stderr_reader = BufReader::new(stderr);

                for line in stderr_reader.lines() {
                    println!("{}", line?);
                }
            };
        }
        Err(e) => return Err(Box::new(e)),
    }

    Ok(())
}
