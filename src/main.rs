use clap::Parser;
use dotenv::dotenv;
use std::env;
use std::fs;
use std::path::PathBuf;

mod opensea;

use opensea::events;

/// CLI tool for fetching NFT data for PyTorch projects.
#[derive(Parser)]
#[command(name = "nftide")]
#[command(about = "Fetch NFT sales and other market data", long_about = None)]
struct Cli {
    /// Unique string to identify a collection on OpenSea. This can be found by visiting the collection on the OpenSea website and noting the last path parameter.
    #[arg(long = "collection-slug")]
    collection_slug: String,

    /// Directory to write the output JSON files
    #[arg(long = "output-path", default_value = "data")]
    output_path: PathBuf,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();
    println!(
        "Fetching NFT sales data for collection: {}",
        cli.collection_slug
    );

    let api_key = env::var("OPENSEA_API_KEY").ok();

    let raw_json = match events::get_sales(&cli.collection_slug, api_key.as_deref()).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error fetching sales: {}", e);
            return;
        }
    };

    if let Err(e) = fs::create_dir_all(&cli.output_path) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    let output_file = cli
        .output_path
        .join(format!("{}_sales.json", cli.collection_slug));

    let formatted_json = match serde_json::from_str::<serde_json::Value>(&raw_json) {
        Ok(val) => serde_json::to_string_pretty(&val).unwrap_or(raw_json.clone()),
        Err(_) => raw_json.clone(),
    };

    if let Err(e) = fs::write(&output_file, &formatted_json) {
        eprintln!("Failed to write output file: {}", e);
        return;
    }
    println!("Sales data written to {}", output_file.display());
}
