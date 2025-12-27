use std::fmt::Display;

/// This function is used for cases where, due to user error, the program reached a critical state and must be aborted.
///
/// Notice this is different than panicking, as a panic indicates an error in the program itself (a correct program
/// must not panic). And it is also different than returning a Result (which implies the error can be handled).
///
/// When in doubt, prefer to return a Result so we can always delegate for the caller to decide whether to handle it
/// or not.
pub fn die(message: impl Display) {
    eprintln!("{message}");
    std::process::exit(1);
}
