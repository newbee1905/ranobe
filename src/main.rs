mod internal;

use std::env;

use ranobe::{
	http::{client_init, fetch_url, CLIENT},
	providers::readlightnovel::ReadLightNovel,
	providers::RanobeScraper,
	utils::open_glow,
};

use crate::internal::select::{select::FuzzySelect, theme::ColorfulTheme};
use surf::{client, Url};

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum RanobeMode {
	#[command(about = "Search and Read Light Novel with glow.")]
	Read,
	#[command(about = "Get latest update list and Read Light Novel with glow.")]
	Latest,
	#[command(about = "Search and Download Light Novel.")]
	Download,
	#[command(about = "Seach and Stash Light Novel with glow.")]
	Stash,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "A scraper to read/download/stash light novels with glow in your terminal.", long_about = None)]
struct Args {
	#[command(subcommand)]
	mode: Option<RanobeMode>,

	/// Provider for anime or light novel.
	#[arg(short = 'r', long, default_value = "readlightnovel")]
	provider: String,

	/// Size of the list. Please only send in positive number.
	#[arg(short, long, default_value_t = 80)]
	wrap: u16,

	/// Size of the list. Please only send in positive number.
	#[arg(short, long, default_value_t = 20)]
	size: usize,
}

#[async_std::main]
async fn main() -> Result<(), surf::Error> {
	let args = Args::parse();

	let mode = match &args.mode {
		None => &RanobeMode::Read,
		Some(m) => m,
	};

	let _ = match mode {
		&RanobeMode::Read => {}
		&RanobeMode::Latest => {}
		&RanobeMode::Stash => {}
		&RanobeMode::Download => {}
	};

	// let selections = vec![
	// 	"Ice Cream",
	// 	"Vanilla Cupcake",
	// 	"Chocolate Muffin",
	// 	"A Pile of sweet, sweet mustard",
	// ];
	//

	let provider = ReadLightNovel::new()?;

	let body = provider.get_latest().await?;

	// println!("{:?}", body);

	let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
		.with_prompt("Choose chapter of light novel to read:")
		.max_length(args.size)
		.default(0)
		.items(&body[..])
		.interact()?;

	let text = match selection {
		Some(i) => provider.get_text(body[i].url.clone()).await?,
		None => "".to_string(),
	};

	open_glow(text, args.wrap)?;

	Ok(())
}
