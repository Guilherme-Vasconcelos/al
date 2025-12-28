use std::fmt::Display;

/// This function is used for cases where, due to user error, the program reached a critical state and must be aborted.
///
/// Notice this is different than panicking, as a panic indicates an error in the program itself (a correct program
/// must not panic). And it is also different than returning a Result (which implies the error can be handled).
///
/// This must be used sparingly. Even if you do not know how to handle an error, it's usually better to return it (and
/// let the caller return again, and again, ..., even if you know it will eventually end up reaching main and cause the
/// program to be aborted) rather than aborting it yourself (allows better unit testing,  causes less API breaking changes
/// if we do decide to handle it differently in the future, etc.)
pub fn die(message: impl Display) {
	eprintln!("{message}");
	std::process::exit(1);
}
