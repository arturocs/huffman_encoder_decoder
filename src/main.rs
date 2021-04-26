use algorithm::*;
use clap::{clap_app, AppSettings};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = clap_app!(huffman_compressor =>
        (version: "0.1")
        (author: "Arturo")
        (about: "Compress and decompress files using Huffman coding")
        (@group action +required =>
            (@arg compress: -c --compress "Compress a file")
            (@arg decompress: -d --decompress "Decompress a file")
        )
        (@arg INPUT: +required "Sets the input file to use")
        (@arg OUTPUT: +required "Sets the output file to use")

    )
    .setting(AppSettings::ArgRequiredElseHelp)
    .get_matches();

    match (m.occurrences_of("compress"), m.occurrences_of("decompress")) {
        (1, 0) => {
            compress_file(m.value_of("INPUT").unwrap(), m.value_of("OUTPUT").unwrap())?;
            println!("File compressed succesfully")
        }
        (0, 1) => {
            decompress_file(m.value_of("INPUT").unwrap(), m.value_of("OUTPUT").unwrap())?;
            println!("File decompressed succesfully")
        }
        _ => panic!("Error parsing arguments"),
    }
    Ok(())
}
