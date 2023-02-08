use crate::internal::select::paging::Paging;
use crate::internal::select::theme::{SimpleTheme, TermThemeRenderer, Theme};
use console::{Key, Term};
use fuzzy_matcher::FuzzyMatcher;
use std::{io, ops::Rem};

use ranobe::providers::Ranobe;

enum InputMode {
	Normal,
	Editing,
}

pub struct FuzzySelect<'a> {
	default: Option<usize>,
	items: Vec<Ranobe>,
	prompt: String,
	report: bool,
	clear: bool,
	highlight_matches: bool,
	max_length: Option<usize>,
	theme: &'a dyn Theme,
	input_mode: &'a InputMode,
	/// Search string that a fuzzy search with start with.
	/// Defaults to an empty string.
	initial_text: String,
}

impl Default for FuzzySelect<'static> {
	fn default() -> Self {
		Self::new()
	}
}

impl FuzzySelect<'static> {
	/// Creates the prompt with a specific text.
	pub fn new() -> Self {
		Self::with_theme(&SimpleTheme)
	}
}

impl FuzzySelect<'_> {
	/// Sets the clear behavior of the menu.
	///
	/// The default is to clear the menu.
	pub fn clear(&mut self, val: bool) -> &mut Self {
		self.clear = val;
		self
	}

	/// Sets a default for the menu
	pub fn default(&mut self, val: usize) -> &mut Self {
		self.default = Some(val);
		self
	}

	/// Add a single item to the fuzzy selector.
	pub fn item(&mut self, item: Ranobe) -> &mut Self {
		self.items.push(item);
		self
	}

	/// Adds multiple items to the fuzzy selector.
	pub fn items(&mut self, items: &[Ranobe]) -> &mut Self {
		for item in items {
			self.items.push(item.clone());
		}
		self
	}

	/// Sets the search text that a fuzzy search starts with.
	pub fn with_initial_text<S: Into<String>>(&mut self, initial_text: S) -> &mut Self {
		self.initial_text = initial_text.into();
		self
	}

	/// Prefaces the menu with a prompt.
	///
	/// When a prompt is set the system also prints out a confirmation after
	/// the fuzzy selection.
	pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Self {
		self.prompt = prompt.into();
		self
	}

	/// Indicates whether to report the selected value after interaction.
	///
	/// The default is to report the selection.
	pub fn report(&mut self, val: bool) -> &mut Self {
		self.report = val;
		self
	}

	/// Indicates whether to highlight matched indices
	///
	/// The default is to highlight the indices
	pub fn highlight_matches(&mut self, val: bool) -> &mut Self {
		self.highlight_matches = val;
		self
	}

	/// Sets an optional max length for a page.
	///
	/// Max length is disabled by None
	pub fn max_length(&mut self, val: usize) -> &mut Self {
		// Paging subtracts two from the capacity, paging does this to
		// make an offset for the page indicator. So to make sure that
		// we can show the intended amount of items we need to add two
		// to our value.
		self.max_length = Some(val + 2);
		self
	}

	/// Enables user interaction and returns the result.
	///
	/// The user interect with the selector with vim-like binding
	///
	/// In Normal Mode, the user can move arround dusing 'k' or 'j' for up and down
	/// along side with arrow keys
	///
	/// In Editing Mode, the user type the fuzzy search and see new result
	///
	/// The user can select the items using 'Enter' and the index of selected item will be returned.
	///
	/// Result contains `Some(index)` if user hit 'Enter' or `None` if user cancelled with 'Esc' or 'q'.
	#[inline]
	pub fn interact(&mut self) -> io::Result<Option<usize>> {
		self.interact_on(&Term::stderr())
	}

	/// Like `interact` but allows a specific terminal to be set.
	#[inline]
	pub fn interact_on(&mut self, term: &Term) -> io::Result<Option<usize>> {
		self._interact_on(term)
	}

	/// Like `interact` but allows a specific terminal to be set.
	fn _interact_on(&mut self, term: &Term) -> io::Result<Option<usize>> {
		// Place cursor at the end of the search term
		let mut position = self.initial_text.len();
		let mut search_term = self.initial_text.to_owned();

		let mut paging = Paging::new(term, self.items.len(), self.max_length);
		let mut render = TermThemeRenderer::new(term, self.theme);
		let mut sel = self.default;

		let mut size_vec = Vec::new();
		for item in self.items.iter().as_slice() {
			let size = &item.title.len();
			size_vec.push(*size);
		}

		// Fuzzy matcher
		let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

		term.hide_cursor()?;

		macro_rules! next_item {
			($filtered_list:expr) => {
				sel = match sel {
					None => Some($filtered_list.len() - 1),
					Some(sel) => Some(
						((sel as i64 - 1 + $filtered_list.len() as i64)
							% ($filtered_list.len() as i64)) as usize,
					),
				};
			};
		}

		macro_rules! prev_item {
			($filtered_list:expr) => {
				sel = match sel {
					None => Some(0),
					Some(sel) => Some((sel as u64 + 1).rem($filtered_list.len() as u64) as usize),
				};
			};
		}

		loop {
			render.clear()?;

			paging.render_prompt(|paging_info| {
				render.fuzzy_select_prompt(
					self.prompt.as_str(),
					&search_term,
					position,
					paging_info,
				)
			})?;

			// Maps all items to a tuple of item and its match score.
			let mut filtered_list = self
				.items
				.iter()
				.map(|item| (item, matcher.fuzzy_match(&item.title, &search_term)))
				.filter_map(|(item, score)| score.map(|s| (item, s)))
				.collect::<Vec<_>>();

			// Renders all matching items, from best match to worst.
			filtered_list.sort_unstable_by(|(_, s1), (_, s2)| s2.cmp(s1));

			for (idx, (item, _)) in filtered_list
				.iter()
				.enumerate()
				.skip(paging.current_page * paging.capacity)
				.take(paging.capacity)
			{
				render.fuzzy_select_prompt_item(
					&item.title,
					Some(idx) == sel,
					self.highlight_matches,
					&matcher,
					&search_term,
				)?;
			}

			term.flush()?;

			match (term.read_key()?, sel) {
				(Key::Escape, _) => match self.input_mode {
					InputMode::Normal => {
						if self.clear {
							render.clear()?;
							term.flush()?;
						}
						term.show_cursor()?;

						return Ok(None);
					}
					InputMode::Editing => self.input_mode = &InputMode::Normal,
				},
				(Key::Char('i'), _) if matches!(self.input_mode, InputMode::Normal) => {
					self.input_mode = &InputMode::Editing
				}
				(Key::ArrowUp | Key::BackTab, _) if !filtered_list.is_empty() => {
					next_item!(filtered_list);
					term.flush()?;
				}
				(Key::Char('k'), _)
					if matches!(self.input_mode, InputMode::Normal)
						&& !filtered_list.is_empty() =>
				{
					next_item!(filtered_list);
					term.flush()?;
				}
				(Key::ArrowDown | Key::Tab, _) if !filtered_list.is_empty() => {
					prev_item!(filtered_list);
					term.flush()?;
				}
				(Key::Char('j'), _)
					if matches!(self.input_mode, InputMode::Normal)
						&& !filtered_list.is_empty() =>
				{
					prev_item!(filtered_list);
					term.flush()?;
				}
				(Key::ArrowLeft, _) if paging.active => sel = Some(paging.previous_page()),
				(Key::Char('h'), _)
					if matches!(self.input_mode, InputMode::Normal) && paging.active =>
				{
					sel = Some(paging.previous_page())
				}
				(Key::ArrowRight, _) if paging.active => sel = Some(paging.next_page()),
				(Key::Char('l'), _)
					if matches!(self.input_mode, InputMode::Normal) && paging.active =>
				{
					sel = Some(paging.next_page())
				}

				(Key::Enter, Some(sel)) => match self.input_mode {
					InputMode::Editing => self.input_mode = &InputMode::Normal,
					InputMode::Normal if !filtered_list.is_empty() => {
						if self.clear {
							render.clear()?;
						}

						if self.report {
							render.input_prompt_selection(
								self.prompt.as_str(),
								filtered_list[sel].0.title.as_str(),
							)?;
						}

						let sel_string = &filtered_list[sel].0.title;
						let sel_string_pos_in_items =
							self.items.iter().position(|item| item.title.eq(sel_string));

						term.show_cursor()?;
						return Ok(sel_string_pos_in_items);
					}
					_ => {}
				},
				(Key::Backspace, _)
					if matches!(self.input_mode, InputMode::Editing) && position > 0 =>
				{
					position -= 1;
					search_term.remove(position);
					term.flush()?;
				}
				(Key::Char(chr), _)
					if matches!(self.input_mode, InputMode::Editing) && !chr.is_ascii_control() =>
				{
					search_term.insert(position, chr);
					position += 1;
					term.flush()?;
					sel = Some(0);
				}

				_ => {}
			}

			match sel {
				Some(sel) => paging.update(sel)?,
				None => paging.update(0)?,
			}

			render.clear_preserve_prompt(&size_vec)?;
		}
	}
}

impl<'a> FuzzySelect<'a> {
	/// Same as `new` but with a specific theme.
	pub fn with_theme(theme: &'a dyn Theme) -> Self {
		Self {
			default: None,
			items: vec![],
			prompt: "".into(),
			report: true,
			clear: true,
			highlight_matches: true,
			max_length: None,
			theme,
			input_mode: &InputMode::Normal,
			initial_text: "".into(),
		}
	}
}
