use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Write},
    iter::FromIterator,
    path::PathBuf,
};

use crate::util::{
    common::EOL,
    err::{err, BfResult},
};

mod cli;
pub use cli::GenerateCli;

mod read;
use read::read_data;

pub fn subcmd_generate(args: GenerateCli) -> BfResult<()> {
    let mut data = read_data(args.infile)?;

    if args.newline && !data.ends_with(EOL) {
        data.push_str(EOL);
    }

    let mut writer: (Box<dyn Write>, PathBuf) = match args.outfile {
        Some(path) => (
            Box::new(
                File::create(&path)
                    .map_err(|e| err!(FileOpen, e, path.clone()))?,
            ),
            path,
        ),
        None => (Box::new(io::stdout()), PathBuf::from("STDOUT")),
    };

    let gen_func = match &*args.mode {
        "charwise" => generator_charwise,
        "linewise" => generator_linewise,
        "unique-chars" => generator_unique_chars,
        _ => Err(format!("invalid mode (impossible): {}", args.mode))?,
    };

    writer
        .0
        .write_all(&gen_func(data).as_bytes())
        .map_err(|e| err!(FileWrite, e, writer.1))
}

fn generator_charwise(data: String) -> String {
    let mut script = String::new();
    gen_loop(&mut script, data.bytes(), true);
    script.push_str(EOL);
    script
}

fn generator_linewise(data: String) -> String {
    let has_final_eol = data.ends_with(EOL);
    let bf_eol = if data.contains("\r\n") {
        "+++++++++++++.---.>" // print "\r\n"
    } else {
        "++++++++++.>" // print "\n"
    };

    let mut script = String::new();

    let lines = data.lines().collect::<Vec<_>>();
    for (i, line) in lines.iter().enumerate() {
        gen_loop(&mut script, line.bytes(), true);
        if i < lines.len() - 1 || has_final_eol {
            script.push_str(bf_eol);
        }
    }

    script.push_str(EOL);
    script
}

fn generator_unique_chars(data: String) -> String {
    // Vec of all unique bytes in data, sorted
    let mut unique_data = HashSet::<u8>::from_iter(data.bytes())
        .into_iter()
        .collect::<Vec<_>>();
    unique_data.sort();

    // HashMap: Key = byte from data, Value = cell index in tape of the byte
    let cell_value_indexes = HashMap::<u8, usize>::from_iter(
        unique_data.iter().enumerate().map(|(i, &b)| (b, i + 1)),
    );

    let mut script = String::new();

    gen_loop(&mut script, unique_data.into_iter(), false);
    script.push('>');
    let mut cursor: usize = 1;

    for c in data.bytes() {
        let cell_index = cell_value_indexes[&c];

        // Difference between cursor position and next byte's cell index
        let diff = cell_index as isize - cursor as isize;

        // Code to move the cursor to the target cell and print it
        let c = if diff < 0 { '<' } else { '>' };
        (0..diff.abs()).for_each(|_| script.push(c));
        script.push('.');

        cursor = cell_index;
    }

    script.push_str(EOL);
    script
}

fn gen_loop<I: Iterator<Item = u8>>(
    script: &mut String,
    data: I,
    print_cells: bool,
) {
    let values = data.collect::<Vec<_>>();
    let approx_values = values
        .iter()
        .cloned()
        .map(|b| 10 * (b as f32 / 10_f32).round() as u8)
        .collect::<Vec<_>>();
    let len = values.len();

    // Loop counter
    script.push_str("++++++++++[");

    // Increment cells to approximated values
    for approx in &approx_values {
        script.push('>');
        let factor = *approx / 10;
        (0..factor).for_each(|_| script.push('+'));
    }

    // Return to counter cell and decrement
    (0..len).for_each(|_| script.push('<'));
    script.push_str("-]");

    // Adjust cells to real value
    for (val, approx) in values.iter().zip(&approx_values) {
        script.push('>');
        let diff = *val as i8 - *approx as i8;
        let c = if diff < 0 { '-' } else { '+' };
        (0..diff.abs()).for_each(|_| script.push(c));
    }

    // Go back to beginning
    (0..len).for_each(|_| script.push('<'));

    if print_cells {
        // Print data
        (0..len).for_each(|_| script.push_str(">."));

        // Move to empty cell
        script.push('>');
    }
}
