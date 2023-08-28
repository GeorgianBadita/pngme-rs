use std::fs;

use clap::Parser;

use crate::chunk::Chunk;
use crate::cli::{Cli, Commands};
use crate::png::Png;

mod cli;
mod chunk;
mod chunk_type;
mod png;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode { file_path, chunk_type, message, output_file } => {
            let file_content = fs::read_to_string(&file_path)?;
            let mut png = Png::try_from(file_content.as_bytes())?;
            let chunk = Chunk::new(chunk_type, message.as_bytes().to_vec());
            png.append_chunk(chunk);
            let out_file = output_file.unwrap_or(file_path);
            fs::write(out_file, png.as_bytes())?
        }
        Commands::Decode { file_path, chunk_type } => {
            let chunk_type_bytes = chunk_type.bytes();
            let chunk_str = std::str::from_utf8(&chunk_type_bytes).unwrap();
            let file_content = fs::read_to_string(file_path)?;
            let png = Png::try_from(file_content.as_bytes())?;
            let chunk_with_message = png.chunk_by_type(chunk_str);
            if let Some(message) = chunk_with_message {
                println!("Message: {}", message.data_as_string()?);
            } else {
                println!("[WARN] - No message found for chunk: {}", chunk_str);
            }
        }
        Commands::Remove { file_path, chunk_type } => {
            let chunk_type_bytes = chunk_type.bytes();
            let chunk_str = std::str::from_utf8(&chunk_type_bytes).unwrap();
            let file_content = fs::read_to_string(file_path)?;
            let mut png = Png::try_from(file_content.as_bytes())?;
            let chunk = png.remove_chunk(chunk_str)?;
            println!("Removed message: {}", chunk.data_as_string()?);
        }
        Commands::Print { file_path } => {
            let file_content = fs::read_to_string(file_path)?;
            let png = Png::try_from(file_content.as_bytes())?;
            png.chunks().iter().for_each(|chunk|
                println!(
                "{}\n-----------", chunk
            ));
        }
    }

    Ok(())
}
