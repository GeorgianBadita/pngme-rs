use clap::{Parser, Subcommand};
use crate::chunk_type::ChunkType;


#[derive(Parser)]
#[command(name = "Pngme")]
#[command(author = "Geo Badita. <geo.badota@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Allows encoding and decoding messages into Png files", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encodes a message into a PNG file
    Encode {
        #[arg(short, long)]
        file_path: String,

        #[arg(short, long)]
        chunk_type: ChunkType,

        #[arg(short, long)]
        message: String,

        #[arg(short, long)]
        output_file: Option<String>
    },
    /// Decodes a message from a PNG file
    Decode {
        #[arg(short, long)]
        file_path: String,

        #[arg(short, long)]
        chunk_type: ChunkType,
    },
    /// Removes a message from a PNG file
    Remove {
        #[arg(short, long)]
        file_path: String,

        #[arg(short, long)]
        chunk_type: ChunkType,
    },
    /// Prints a list of PNG chunks that can be searched for messages
    Print {
        #[arg(short, long)]
        file_path: String,
    },
}