//! Customizes the rendering of the elements.
use std::{fmt, io};

use console::{measure_text_width, style, Style, StyledObject, Term};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

/// Implements a theme for dialoguer.
pub trait Theme {
	/// Formats a prompt.
	#[inline]
	fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
		write!(f, "{}:", prompt)
	}

	/// Formats out an error.
	#[inline]
	fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
		write!(f, "error: {}", err)
	}

	/// Formats an input prompt.
	fn format_input_prompt(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		default: Option<&str>,
	) -> fmt::Result {
		match default {
			Some(default) if prompt.is_empty() => write!(f, "[{}]: ", default),
			Some(default) => write!(f, "{} [{}]: ", prompt, default),
			None => write!(f, "{}: ", prompt),
		}
	}

	/// Formats an input prompt after selection.
	#[inline]
	fn format_input_prompt_selection(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		sel: &str,
	) -> fmt::Result {
		write!(f, "{}: {}", prompt, sel)
	}

	/// Formats a fuzzy select prompt item.
	fn format_fuzzy_select_prompt_item(
		&self,
		f: &mut dyn fmt::Write,
		text: &str,
		active: bool,
		highlight_matches: bool,
		matcher: &SkimMatcherV2,
		search_term: &str,
	) -> fmt::Result {
		write!(f, "{} ", if active { ">" } else { " " })?;

		if highlight_matches {
			if let Some((_score, indices)) = matcher.fuzzy_indices(text, &search_term) {
				for (idx, c) in text.chars().into_iter().enumerate() {
					if indices.contains(&idx) {
						write!(f, "{}", style(c).for_stderr().bold())?;
					} else {
						write!(f, "{}", c)?;
					}
				}

				return Ok(());
			}
		}

		write!(f, "{}", text)
	}

	/// Formats a fuzzy select prompt.
	fn format_fuzzy_select_prompt(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		search_term: &str,
		cursor_pos: usize,
	) -> fmt::Result {
		if !prompt.is_empty() {
			write!(f, "{} ", prompt,)?;
		}

		if cursor_pos < search_term.len() {
			let st_head = search_term[0..cursor_pos].to_string();
			let st_tail = search_term[cursor_pos..search_term.len()].to_string();
			let st_cursor = "|".to_string();
			write!(f, "{}{}{}", st_head, st_cursor, st_tail)
		} else {
			let cursor = "|".to_string();
			write!(f, "{}{}", search_term.to_string(), cursor)
		}
	}
}

/// The default theme.
pub struct SimpleTheme;

impl Theme for SimpleTheme {}

/// A colorful theme
pub struct ColorfulTheme {
	/// The style for default values
	pub defaults_style: Style,
	/// The style for prompt
	pub prompt_style: Style,
	/// Prompt prefix value and style
	pub prompt_prefix: StyledObject<String>,
	/// Prompt suffix value and style
	pub prompt_suffix: StyledObject<String>,
	/// Prompt on success prefix value and style
	pub success_prefix: StyledObject<String>,
	/// Prompt on success suffix value and style
	pub success_suffix: StyledObject<String>,
	/// Error prefix value and style
	pub error_prefix: StyledObject<String>,
	/// The style for error message
	pub error_style: Style,
	/// The style for hints
	pub hint_style: Style,
	/// The style for values on prompt success
	pub values_style: Style,
	/// The style for active items
	pub active_item_style: Style,
	/// The style for inactive items
	pub inactive_item_style: Style,
	/// Active item in select prefix value and style
	pub active_item_prefix: StyledObject<String>,
	/// Inctive item in select prefix value and style
	pub inactive_item_prefix: StyledObject<String>,
	/// Checked item in multi select prefix value and style
	pub checked_item_prefix: StyledObject<String>,
	/// Unchecked item in multi select prefix value and style
	pub unchecked_item_prefix: StyledObject<String>,
	/// Picked item in sort prefix value and style
	pub picked_item_prefix: StyledObject<String>,
	/// Unpicked item in sort prefix value and style
	pub unpicked_item_prefix: StyledObject<String>,
	/// Formats the cursor for a fuzzy select prompt
	pub fuzzy_cursor_style: Style,
	// Formats the highlighting if matched characters
	pub fuzzy_match_highlight_style: Style,
	/// Show the selections from certain prompts inline
	pub inline_selections: bool,
}

impl Default for ColorfulTheme {
	fn default() -> ColorfulTheme {
		ColorfulTheme {
			defaults_style: Style::new().for_stderr().cyan(),
			prompt_style: Style::new().for_stderr().bold(),
			prompt_prefix: style("?".to_string()).for_stderr().yellow(),
			prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
			success_prefix: style("✔".to_string()).for_stderr().green(),
			success_suffix: style("·".to_string()).for_stderr().black().bright(),
			error_prefix: style("✘".to_string()).for_stderr().red(),
			error_style: Style::new().for_stderr().red(),
			hint_style: Style::new().for_stderr().black().bright(),
			values_style: Style::new().for_stderr().green(),
			active_item_style: Style::new().for_stderr().cyan(),
			inactive_item_style: Style::new().for_stderr(),
			active_item_prefix: style("❯".to_string()).for_stderr().green(),
			inactive_item_prefix: style(" ".to_string()).for_stderr(),
			checked_item_prefix: style("✔".to_string()).for_stderr().green(),
			unchecked_item_prefix: style("✔".to_string()).for_stderr().black(),
			picked_item_prefix: style("❯".to_string()).for_stderr().green(),
			unpicked_item_prefix: style(" ".to_string()).for_stderr(),
			fuzzy_cursor_style: Style::new().for_stderr().black().on_white(),
			fuzzy_match_highlight_style: Style::new().for_stderr().bold(),
			inline_selections: true,
		}
	}
}

impl Theme for ColorfulTheme {
	/// Formats a prompt.
	fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
		if !prompt.is_empty() {
			write!(
				f,
				"{} {} ",
				&self.prompt_prefix,
				self.prompt_style.apply_to(prompt)
			)?;
		}

		write!(f, "{}", &self.prompt_suffix)
	}

	/// Formats an error
	fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
		write!(
			f,
			"{} {}",
			&self.error_prefix,
			self.error_style.apply_to(err)
		)
	}

	/// Formats an input prompt.
	fn format_input_prompt(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		default: Option<&str>,
	) -> fmt::Result {
		if !prompt.is_empty() {
			write!(
				f,
				"{} {} ",
				&self.prompt_prefix,
				self.prompt_style.apply_to(prompt)
			)?;
		}

		match default {
			Some(default) => write!(
				f,
				"{} {} ",
				self.hint_style.apply_to(&format!("({})", default)),
				&self.prompt_suffix
			),
			None => write!(f, "{} ", &self.prompt_suffix),
		}
	}

	/// Formats an input prompt after selection.
	fn format_input_prompt_selection(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		sel: &str,
	) -> fmt::Result {
		if !prompt.is_empty() {
			write!(
				f,
				"{} {} ",
				&self.success_prefix,
				self.prompt_style.apply_to(prompt)
			)?;
		}

		write!(
			f,
			"{} {}",
			&self.success_suffix,
			self.values_style.apply_to(sel)
		)
	}

	/// Formats a fuzzy select prompt item.
	fn format_fuzzy_select_prompt_item(
		&self,
		f: &mut dyn fmt::Write,
		text: &str,
		active: bool,
		highlight_matches: bool,
		matcher: &SkimMatcherV2,
		search_term: &str,
	) -> fmt::Result {
		write!(
			f,
			"{} ",
			if active {
				&self.active_item_prefix
			} else {
				&self.inactive_item_prefix
			}
		)?;

		if highlight_matches {
			if let Some((_score, indices)) = matcher.fuzzy_indices(text, &search_term) {
				for (idx, c) in text.chars().into_iter().enumerate() {
					if indices.contains(&idx) {
						if active {
							write!(
								f,
								"{}",
								self.active_item_style
									.apply_to(self.fuzzy_match_highlight_style.apply_to(c))
							)?;
						} else {
							write!(f, "{}", self.fuzzy_match_highlight_style.apply_to(c))?;
						}
					} else {
						if active {
							write!(f, "{}", self.active_item_style.apply_to(c))?;
						} else {
							write!(f, "{}", c)?;
						}
					}
				}

				return Ok(());
			}
		}

		write!(f, "{}", text)
	}

	/// Formats a fuzzy-selectprompt after selection.
	fn format_fuzzy_select_prompt(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		search_term: &str,
		cursor_pos: usize,
	) -> fmt::Result {
		if !prompt.is_empty() {
			write!(
				f,
				"{} {} ",
				&self.prompt_prefix,
				self.prompt_style.apply_to(prompt)
			)?;
		}

		if cursor_pos < search_term.len() {
			let st_head = search_term[0..cursor_pos].to_string();
			let st_tail = search_term[cursor_pos + 1..search_term.len()].to_string();
			let st_cursor = self
				.fuzzy_cursor_style
				.apply_to(search_term.to_string().chars().nth(cursor_pos).unwrap());
			write!(
				f,
				"{} {}{}{}",
				&self.prompt_suffix, st_head, st_cursor, st_tail
			)
		} else {
			let cursor = self.fuzzy_cursor_style.apply_to(" ");
			write!(
				f,
				"{} {}{}",
				&self.prompt_suffix,
				search_term.to_string(),
				cursor
			)
		}
	}
}

/// Helper struct to conveniently render a theme of a term.
pub(crate) struct TermThemeRenderer<'a> {
	term: &'a Term,
	theme: &'a dyn Theme,
	height: usize,
	prompt_height: usize,
	prompts_reset_height: bool,
}

impl<'a> TermThemeRenderer<'a> {
	pub fn new(term: &'a Term, theme: &'a dyn Theme) -> TermThemeRenderer<'a> {
		TermThemeRenderer {
			term,
			theme,
			height: 0,
			prompt_height: 0,
			prompts_reset_height: true,
		}
	}

	pub fn add_line(&mut self) {
		self.height += 1;
	}

	fn write_formatted_str<
		F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
	>(
		&mut self,
		f: F,
	) -> io::Result<usize> {
		let mut buf = String::new();
		f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
		self.height += buf.chars().filter(|&x| x == '\n').count();
		self.term.write_str(&buf)?;
		Ok(measure_text_width(&buf))
	}

	fn write_formatted_line<
		F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
	>(
		&mut self,
		f: F,
	) -> io::Result<()> {
		let mut buf = String::new();
		f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
		self.height += buf.chars().filter(|&x| x == '\n').count() + 1;
		self.term.write_line(&buf)
	}

	fn write_formatted_prompt<
		F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
	>(
		&mut self,
		f: F,
	) -> io::Result<()> {
		self.write_formatted_line(f)?;
		if self.prompts_reset_height {
			self.prompt_height = self.height;
			self.height = 0;
		}
		Ok(())
	}

	fn write_paging_info(buf: &mut dyn fmt::Write, paging_info: (usize, usize)) -> fmt::Result {
		write!(buf, " [Page {}/{}] ", paging_info.0, paging_info.1)
	}

	pub fn error(&mut self, err: &str) -> io::Result<()> {
		self.write_formatted_line(|this, buf| this.theme.format_error(buf, err))
	}

	pub fn fuzzy_select_prompt(
		&mut self,
		prompt: &str,
		search_term: &str,
		cursor_pos: usize,
		paging_info: Option<(usize, usize)>,
	) -> io::Result<()> {
		self.write_formatted_prompt(|this, buf| {
			if let Some(paging_info) = paging_info {
				TermThemeRenderer::write_paging_info(buf, paging_info)?;
			}

			this.theme
				.format_fuzzy_select_prompt(buf, prompt, search_term, cursor_pos)?;

			Ok(())
		})
	}

	pub fn input_prompt(&mut self, prompt: &str, default: Option<&str>) -> io::Result<usize> {
		self.write_formatted_str(|this, buf| this.theme.format_input_prompt(buf, prompt, default))
	}

	pub fn input_prompt_selection(&mut self, prompt: &str, sel: &str) -> io::Result<()> {
		self.write_formatted_prompt(|this, buf| {
			this.theme.format_input_prompt_selection(buf, prompt, sel)
		})
	}

	pub fn fuzzy_select_prompt_item(
		&mut self,
		text: &str,
		active: bool,
		highlight: bool,
		matcher: &SkimMatcherV2,
		search_term: &str,
	) -> io::Result<()> {
		self.write_formatted_line(|this, buf| {
			this.theme.format_fuzzy_select_prompt_item(
				buf,
				text,
				active,
				highlight,
				matcher,
				search_term,
			)
		})
	}

	pub fn clear(&mut self) -> io::Result<()> {
		self.term
			.clear_last_lines(self.height + self.prompt_height)?;
		self.height = 0;
		self.prompt_height = 0;
		Ok(())
	}

	pub fn clear_preserve_prompt(&mut self, size_vec: &[usize]) -> io::Result<()> {
		let mut new_height = self.height;
		let prefix_width = 2;
		//Check each item size, increment on finding an overflow
		for size in size_vec {
			if *size > self.term.size().1 as usize {
				new_height += (((*size as f64 + prefix_width as f64) / self.term.size().1 as f64)
					.ceil()) as usize - 1;
			}
		}

		self.term.clear_last_lines(new_height)?;
		self.height = 0;
		Ok(())
	}
}
