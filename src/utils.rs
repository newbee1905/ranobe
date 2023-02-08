use std::io::Result;
use std::process::{Command, ExitStatus, Stdio};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
	static ref STRING_RE: Regex =
		Regex::new(r#"(“|"|&quot;|&ldquo;)(.+?)(”|"|&quot;|&rdquo;)"#).unwrap();
}

pub fn italicize(text: &String) -> String {
	STRING_RE.replace_all(text, " _${1}${2}${3}_ ").to_string()
}

pub fn open_glow(text: String, wrap: u16) -> Result<ExitStatus> {
	let termsize::Size { rows: _, cols } = termsize::get().unwrap();

	let cols = std::cmp::min(cols, wrap);

	let echo = Command::new("echo")
		.arg(text)
		.stdout(Stdio::piped())
		.spawn()?;

	let sorf_wrap = Command::new("fold")
		.arg("-s")
		.arg("-w")
		.arg(cols.to_string())
		.stdin(Stdio::from(echo.stdout.unwrap()))
		.stdout(Stdio::piped())
		.spawn()?;

	Command::new("glow")
		.arg("-p")
		.arg("-w")
		.arg((cols + 1).to_string())
		.stdin(Stdio::from(sorf_wrap.stdout.unwrap()))
		.spawn()?
		.wait()

	// Command::new("mdless")
	// 	.arg("--columns")
	// 	.arg((cols + 1).to_string())
	// 	.stdin(Stdio::from(echo.stdout.unwrap()))
	// 	.spawn()?
	// 	.wait()
}
