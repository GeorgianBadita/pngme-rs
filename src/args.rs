use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(name = "Pngme")]
#[command(author = "Geo Badita. <geo.badota@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Allows encoding and decoding messages into Png files", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encodes a message into a PNG file
    Encode {

    },
    /// Decodes a message from a PNG file
    Decode {},
    /// Removes a message from a PNG file
    Remove {},
    /// Prints
    Print {},
}