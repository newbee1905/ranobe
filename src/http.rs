use lazy_static::lazy_static;
use std::time::Duration;
use surf::Url;
use surf::{Client, Config};

use once_cell::sync::OnceCell;

lazy_static! {
	static ref USER_AGENT: &'static str =
		"Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
	pub static ref CLIENT: OnceCell<Client> = OnceCell::new();
}

pub fn client_init() -> Result<Client, surf::Error> {
	Ok(<Config as TryInto<Client>>::try_into(
		Config::new()
			.set_timeout(Some(Duration::from_secs(30)))
			.add_header("user-agent", *USER_AGENT)?,
	)?
	.with(surf::middleware::Redirect::default()))
}

pub async fn fetch_url(client: &Client, url: Url) -> Result<String, surf::Error> {
	client.get(url).recv_string().await
}
