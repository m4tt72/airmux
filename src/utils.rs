use console::Term;
use shell_words::{quote, split};
use snafu::{ensure, Snafu};
use std::error;
use std::path;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Project name {:?} cannot not have a trailing slash", project_name))]
    ProjectNameTrailingSlash { project_name: String },
    #[snafu(display("Project name {:?} cannot not be an absolute path", project_name))]
    ProjectNameAbsolutePath { project_name: String },
    #[snafu(display("Command cannot be empty"))]
    EmptyCommand {},
    #[snafu(display("name {:?} cannot contain the following characters: .: ", identifier))]
    TmuxIdentifierIllegalCharacters { identifier: String },
    #[snafu(display("name cannot be empty"))]
    TmuxIdentifierEmpty {},
}

pub fn valid_tmux_identifier(identifier: &str) -> Result<(), Box<dyn error::Error>> {
    ensure!(
        identifier.find(&['.', ':'][..]).is_none(),
        TmuxIdentifierIllegalCharacters { identifier }
    );
    ensure!(!identifier.is_empty(), TmuxIdentifierEmpty {});

    Ok(())
}

pub fn get_project_namespace(project_name: &str) -> Result<PathBuf, Box<dyn error::Error>> {
    let has_trailing_slash = project_name.ends_with(path::MAIN_SEPARATOR);
    ensure!(
        !has_trailing_slash,
        ProjectNameTrailingSlash { project_name }
    );

    let path = PathBuf::from(project_name);
    ensure!(!path.has_root(), ProjectNameAbsolutePath { project_name });

    Ok(path.parent().unwrap().to_path_buf())
}

pub fn parse_command(
    command: &str,
    args: &[&str],
) -> Result<(String, Vec<String>), Box<dyn error::Error>> {
    ensure!(!command.is_empty(), EmptyCommand {});

    let args_iter = args.to_owned().into_iter().map(String::from);
    let mut command_parts = split(command)?.into_iter().chain(args_iter);

    let new_command = command_parts.next().unwrap();
    let new_args: Vec<String> = command_parts.collect();
    Ok((new_command, new_args))
}

pub fn is_default<T>(t: &T) -> bool
where
    T: Default + PartialEq,
{
    t == &T::default()
}

pub fn prompt_confirmation(message: &str, default: bool) -> Result<bool, Box<dyn error::Error>> {
    let reply_hint = if default { "Y/n" } else { "y/N" };

    // Use the unbuffered stdout to print the prompt
    let term = Term::stdout();
    term.write_str(&format!("{} ({}): ", message, reply_hint))?;

    // Get reply
    let reply = term.read_char()?;
    let reply = if reply == '\n' {
        default
    } else {
        reply.to_ascii_lowercase() == 'y'
    };

    // Type out the reply before returning
    term.write_line(if reply { "y" } else { "n" })?;

    Ok(reply)
}

pub fn tmux_quote(part: &str) -> String {
    quote(part).replace("'\\''", "'\"'\"'")
}

pub fn tmux_join(parts: &[&str]) -> String {
    let parts: Vec<String> = parts.to_owned().into_iter().map(tmux_quote).collect();
    parts.join(" ")
}

#[cfg(test)]
#[path = "test/utils.rs"]
mod tests;
