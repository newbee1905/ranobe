use surf::utils::async_trait;
use surf::Url;

pub mod readlightnovel;

#[derive(Debug, Clone)]
pub struct Ranobe {
	pub title: String,
	pub url: Url,
}

#[async_trait]
pub trait RanobeScraper {
	async fn get_latest(&mut self) -> Result<Vec<Ranobe>, surf::Error>;
	async fn get_next_page(id: &str, page: &u32) -> Result<String, surf::Error>;
	async fn get_prev_page(id: &str, page: &u32) -> Result<String, surf::Error>;
	async fn get_list(html: &str) -> Result<String, surf::Error>;
	async fn get_text(&self, url: Url) -> Result<String, surf::Error>;
}

impl Ranobe {
	pub async fn new(title: String, url: &str) -> Result<Self, surf::Error> {
		Ok(Self {
			title,
			url: Url::parse(url)?,
		})
	}
}
