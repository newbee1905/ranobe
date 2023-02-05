mod internal;

use crate::internal::select::{select::FuzzySelect, theme::ColorfulTheme};
// use dialoguer::{console::Term, theme::ColorfulTheme, FuzzySelect};

use clap::{Parser, Subcommand};
use std::thread;

#[derive(Subcommand, Debug)]
enum RanobeMode {
	#[command(about = "Read Light Novel with glow.")]
	Read,
	#[command(about = "Download Light Novel.")]
	Download,
	#[command(about = "Stash Light Novel with glow.")]
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
	#[arg(short, long, default_value_t = 20)]
	size: usize,
}

fn main() -> std::io::Result<()> {
	let args = Args::parse();

	let mode = match &args.mode {
		None => &RanobeMode::Read,
		Some(m) => m,
	};

	let _ = match mode {
		&RanobeMode::Read => {}
		&RanobeMode::Stash => {}
		&RanobeMode::Download => {}
	};

	let selections = vec![
		"Ice Cream",
		"Vanilla Cupcake",
		"Chocolate Muffin",
		"A Pile of sweet, sweet mustard",
	];

	let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
		.with_prompt("Pick your flavor")
		.max_length(args.size)
		.default(0)
		.items(&selections[..])
		.interact()?;

	match selection {
		Some(i) => println!("Enjoy your {}!", selections[i]),
		None => println!("You didnt select anything"),
	}

	Ok(())
}
