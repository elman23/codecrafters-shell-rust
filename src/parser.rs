/// Represents a parsed command with its name and arguments.
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    /// The raw string after the command name (preserved for builtins that need it).
    pub raw_args: String,
}

impl Command {
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let name = parse_command_name(input);
        let raw_args = input[name.len()..].trim_start().to_string();
        let clean_name = strip_quotes(&name);
        let args = if raw_args.is_empty() {
            vec![]
        } else {
            parse_args(&raw_args)
        };

        Some(Command { name: clean_name, args, raw_args })
    }
}

/// Extract the command name token (respects quotes).
fn parse_command_name(s: &str) -> String {
    let mut in_double = false;
    let mut in_single = false;
    let mut token = String::new();

    for c in s.chars() {
        match c {
            ' ' if !in_double && !in_single => break,
            '"' => in_double = !in_double,
            '\'' => in_single = !in_single,
            _ => {}
        }
        token.push(c);
    }

    token
}

/// Remove surrounding/embedded quotes and handle escape sequences.
pub fn strip_quotes(s: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut out = String::new();

    for c in s.chars() {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_double => escaped = true,
            '\\' if !in_single => escaped = true,
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            _ => out.push(c),
        }
    }

    out
}

/// Split arguments respecting quotes and escape sequences.
pub fn parse_args(input: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' if in_double => escaped = true,
            '\\' if !in_single => escaped = true,
            '"' if !in_single => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            ' ' if !in_double && !in_single => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Parse echo arguments (collapses extra spaces outside quotes).
pub fn parse_echo_args(input: &str) -> String {
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;
    let mut result = String::new();

    // Strip empty quote pairs first
    let input = input.replace("\"\"", "").replace("''", "");

    for c in input.chars() {
        if escaped {
            result.push(c);
            escaped = false;
            continue;
        }
        match c {
            '\\' if !in_single => escaped = true,
            '"' if !in_single => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            ' ' if !in_double && !in_single => {
                if !result.ends_with(' ') {
                    result.push(' ');
                }
            }
            _ => result.push(c),
        }
    }

    result.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let cmd = Command::parse("echo hello world").unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_quoted_args() {
        let cmd = Command::parse(r#"echo "hello world""#).unwrap();
        assert_eq!(cmd.args, vec!["hello world"]);
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("'hello'"), "hello");
        assert_eq!(strip_quotes(r#""hello""#), "hello");
    }
}