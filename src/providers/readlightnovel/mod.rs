use crate::{
	http::{client_init, fetch_url, CLIENT},
	utils::italicize,
};
use std::fmt::format;
use surf::utils::async_trait;

use lazy_static::lazy_static;
use regex::Regex;
use surf::Client;
use surf::Url;

use super::{Ranobe, RanobeScraper};

lazy_static! {
	static ref LATEST_RE: Regex =
		Regex::new(r#"<a itemprop="url" href="(.+)" rel="bookmark">(.+)</a>"#).unwrap();
	static ref TITLE_RE: Regex = Regex::new(r#"<h1><a .+?>(.+?)<\/a>(.+?)<\/h1>"#).unwrap();
	static ref TEXT_RE: Regex = Regex::new(r#"<p>(.+?)</p>"#).unwrap();
	static ref RAW_TEXT_RE: Regex =
		Regex::new(r#"<!-- audio -->[\S\s]+?<!-- audio -->([\S\s]+?)<!-- .+ desktop start -->"#)
			.unwrap();
	static ref BREAK_RE: Regex = Regex::new("<br>").unwrap();
}

#[derive(Debug)]
pub struct ReadLightNovel {
	// client: Client,
	page: u32,
}

impl ReadLightNovel {
	pub fn new() -> Result<Self, surf::Error> {
		Ok(Self {
			// client: client_init()?,
			page: 0,
		})
	}
}

#[async_trait]
impl RanobeScraper for ReadLightNovel {
	async fn get_latest(&mut self) -> Result<Vec<Ranobe>, surf::Error> {
		let client = CLIENT.get_or_init(|| client_init().unwrap());

		let body = fetch_url(
			&client,
			Url::parse(&*format!(
				"https://readlightnovel.me/latest-update/{}",
				self.page
			))?,
		)
		.await?;

		let mut ranobe_list: Vec<Ranobe> = Vec::new();
		for ranobe in LATEST_RE.captures_iter(&*body) {
			let url = ranobe.get(1).unwrap().as_str().trim();
			let title = ranobe.get(2).unwrap().as_str().trim().to_string();
			ranobe_list.push(Ranobe::new(title, url).await?);
		}

		self.page += 1;

		Ok(ranobe_list)
	}
	async fn get_next_page(id: &str, page: &u32) -> Result<String, surf::Error> {
		Ok(String::new())
	}
	async fn get_prev_page(id: &str, page: &u32) -> Result<String, surf::Error> {
		Ok(String::new())
	}
	async fn get_list(html: &str) -> Result<String, surf::Error> {
		Ok(String::new())
	}
	async fn get_text(&self, url: Url) -> Result<String, surf::Error> {
		let client = CLIENT.get_or_init(|| client_init().unwrap());

		let body = fetch_url(&client, url).await?;

		let title = TITLE_RE.captures(body.as_str()).unwrap().get(1).unwrap();

		let mut _text = String::new();

		let _text = RAW_TEXT_RE
			.captures_iter(&*body)
			.fold(String::new(), |acc, cap| {
				format!("{}{}", _text, cap.get(1).unwrap().as_str().trim())
			});

		// Only get block content
		let mut text = TEXT_RE
			.captures_iter(_text.as_str())
			.fold(String::new(), |acc, cap| {
				format!("{}{}\n", acc, cap.get(1).unwrap().as_str())
			});

		if text.is_empty() {
			text = _text
		}

		// Highlight text inside double quotes
		let text = italicize(&text);

		// Convert all <br> into \n
		let text = BREAK_RE.replace_all(&*text, "\n").to_string();

		Ok(text)
	}
}
