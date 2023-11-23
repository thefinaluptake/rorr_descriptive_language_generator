use std::io::{Write, BufWriter, Seek};
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use clap::Parser;
use anyhow::{anyhow, Context, Result};
use serde_json_lenient::Value;

#[derive(Parser, Debug)]
#[command(author = "thefinaluptake", version, about = "Generates a new language that makes the pickup description the logbook description for Risk of Rain Returns", long_about = None)]
struct Args {
    #[arg(help = "The language folder to generate a copy of and modify")]
    input_language: PathBuf,
    #[arg(help = "The output folder for the new language, if left empty will default to be located beside the input folder and named [input folder name]_desc")]
    output_location: Option<PathBuf>
}

fn main() -> Result<()> {
    const NEW_LINE_THRESHOLD: usize = 80;

    let args = Args::parse();

    let input_language = args.input_language;
    let output_location = args.output_location;

    if !input_language.is_dir() {
        return Err(anyhow!("Input language must be a directory"));
    }

    let language_file = File::open(input_language.join("lang.json")).context("Given input language does not contain a lang.json file")?;

    let mut language: Value = serde_json_lenient::from_reader(language_file).context("Could not load language's lang.json as JSON")?;

    let Some(items) = language["item"].as_object_mut() else {
        return Err(anyhow!("Input language does not contain an object list of items"))
    };

    for (_, attributes) in items.iter_mut() {
        let Some(description) = attributes["description"].as_str() else {
            continue;
        };

        let description = description.to_owned();

        let description = textwrap::fill(&description, NEW_LINE_THRESHOLD);

        let pickup_text = &mut attributes["pickup"];

        if !pickup_text.is_string() {
            continue;
        }

        *pickup_text = Value::String(description);
    }

    let output_location = output_location.unwrap_or_else(|| {
        let mut output_name = input_language.clone();
        let output_name = output_name.as_mut_os_string();
        output_name.push("_desc");

        PathBuf::from(output_name.to_owned())
    });

    println!("{output_location:?}");

    std::fs::create_dir(&output_location).context("Could not create output folder")?;

    std::fs::copy(input_language.join("icon.png"), output_location.join("icon.png")).context("Could not copy input language files to output folder")?;
    std::fs::copy(input_language.join("name.txt"), output_location.join("name.txt")).context("Could not copy input language files to output folder")?;

    let mut name_file = OpenOptions::new().write(true).open(output_location.join("name.txt")).context("Could not open name file")?;
    name_file.seek(std::io::SeekFrom::End(0)).context("Could not seek to end of name file")?;
    write!(&mut name_file, "_desc").context("Could not write name file")?;

    let output_lang = File::create(output_location.join("lang.json")).context("Could not create lang file in output folder")?;

    let mut output_writer = BufWriter::new(output_lang);

    serde_json_lenient::to_writer(&mut output_writer, &language)?;

    output_writer.flush()?;

    Ok(())
}
