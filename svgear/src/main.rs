use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use svgear::painter::{MathjaxServer, Mermaid, PaintParams};
use svgear::{PaintType, Painter, RenderRequest, SharedSvgManager};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    exe_path: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate SVG from MathJax
    Render {
        /// input content
        input: String,
        #[arg(short, long)]
        input_type: String,
        #[arg(short, long)]
        output_type: String,
        #[arg(short, long)]
        width: Option<u32>,
        #[arg(short, long)]
        height: Option<u32>,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Run in server mode
    Serve {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

 /// Get input content from string or file
 fn get_input_content(input: &str) -> Result<String> {
     // Check if input is a file path
     if Path::new(input).exists() {
         fs::read_to_string(input).context("Failed to read input file")
     } else {
         // Use the input directly
         Ok(input.to_string())
     }
 }

 #[tokio::main]
 async fn main() -> Result<()> {
     let cli = Cli::parse();

     match cli.command {
         Commands::Render { input, input_type, output_type, width, height, output } => {
             // Get content from input string or file
             let content = match input_type.as_str() {
                 "inlinetex" => input.clone(), // Use directly for inline TeX
                 _ => get_input_content(&input)?, // Try to load from file for others
             };

             match input_type.as_str() {
                 "svg" => {
                     // Direct SVG rendering
                     let mut manager = svgear::SvgManager::new();
                     let resp = manager.process_render_request(RenderRequest {
                         svg_data: content.clone(),
                         width,
                         height,
                         id: None,
                     })?;

                     // Output based on requested format
                     if output_type == "png" {
                         // Write bitmap data to file or stdout
                         if let Some(output) = output {
                             fs::write(&output, &resp.bitmap.data)
                                 .context("Failed to write output file")?;
                             println!("Saved PNG to {}", output);
                         } else {
                             let mut stdout = std::io::stdout();
                             stdout.write_all(&resp.bitmap.data)?;
                         }
                     } else if output_type == "svg" {
                         // Write SVG to file or stdout
                         if let Some(output) = output {
                             fs::write(&output, content)
                                 .context("Failed to write output file")?;
                             println!("Saved SVG to {}", output);
                         } else {
                             println!("{}", content);
                         }
                     }
                 },
                 "mermaid" | "inlinetex" | "equation" => {
                     // Create a painter with MathJax server
                     let mathjax = MathjaxServer::new(cli.exe_path);
                     let mermaid = Mermaid::new();
                     let painter = Painter::new();

                     // Determine paint type
                     let paint_type = match input_type.as_str() {
                         "mermaid" => PaintType::Mermaid,
                         "inlinetex" => PaintType::InlineTeX,
                         "equation" => PaintType::Equation,
                         _ => unreachable!(),
                     };



                     // Create paint params
                     let params = PaintParams {
                         content,
                         ty: paint_type,
                     };

                     // Paint to SVG
                     let svg_content = painter.paint(params).await?;

                     if output_type == "svg" {
                         // Output SVG directly
                         if let Some(output) = output {
                             fs::write(&output, svg_content)
                                 .context("Failed to write output file")?;
                             println!("Saved SVG to {}", output);
                         } else {
                             println!("{}", svg_content);
                         }
                     } else if output_type == "png" {
                         // Render SVG to bitmap
                         let mut manager = svgear::SvgManager::new();
                         let resp = manager.process_render_request(RenderRequest {
                             svg_data: svg_content,
                             width,
                             height,
                             id: None,
                         })?;

                         // Write bitmap
                         if let Some(output) = output {
                             fs::write(&output, &resp.bitmap.data)
                                 .context("Failed to write output file")?;
                             println!("Saved PNG to {}", output);
                         } else {
                             let mut stdout = std::io::stdout();
                             stdout.write_all(&resp.bitmap.data)?;
                         }
                     }
                 },
                 _ => {
                     return Err(anyhow::anyhow!("Unsupported input type: {}", input_type));
                 }
             }
         },
         Commands::Serve { port } => {
             svgear::run_server(port, cli.exe_path).await?;
         }
     }

     Ok(())
 }
