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
        Err(e) => println!("error : {:#?}", e.to_string()),
    }
}

fn run_command(line: String) -> Result<(), Box<dyn std::error::Error>> {
    let commands = tokenize(&line);
    let mut commands = commands.into_iter();
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
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote_char = None;
    let mut escape = false;

    for ch in input.chars() {
        match ch {
            '\\' => {
                if escape {
                    token.push(ch);
                    escape = false;
                } else {
                    escape = true;
                }
            }
            '"' | '\'' => {
                if escape {
                    token.push(ch);
                    escape = false;
                } else if quote_char == Some(ch) {
                    quote_char = None;
                    tokens.push(token);
                    token = String::new();
                } else if quote_char.is_none() {
                    quote_char = Some(ch);
                } else {
                    token.push(ch);
                }
            }
            ' ' | '\t' if quote_char.is_none() => {
                if !token.is_empty() {
                    tokens.push(token);
                    token = String::new();
                }
                escape = false;
            }
            _ => {
                if escape {
                    escape = false;
                }
                token.push(ch);
            }
        }
    }

    if !token.is_empty() {
        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {

    use crate::tokenize;

    #[test]
    fn test_gpt() {
        let input =
            r#"This is 'a test' of "quoted strings" and unquoted strings. \"escaped quotes\""#;

        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                "This",
                "is",
                "a test",
                "of",
                "quoted strings",
                "and",
                "unquoted",
                "strings.",
                r#""escaped"#,
                r#"quotes""#
            ]
        );
    }
}
