use anyhow::Context;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::structs::OperationStatus;

pub(crate) fn generate_json_string(status: &OperationStatus) -> Result<(), anyhow::Error> {
    let filename = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Errored when calculating time since UNIX_EPOCH")?
        .as_secs();
    let filepath = std::env::current_dir()?;
    let filepath = filepath.join(format!("mvr_job_{filename}.json"));
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(filepath)?;

    let mut writer = BufWriter::new(file);
    writeln!(writer, "[")?;
    let mut iterator = (0..status.status.len()).peekable();
    while let Some(i) = iterator.next() {
        write!(
            writer,
            "    {{ \"source\":\"{}\", \"destination\":\"{}\", \"status\":\"{}\", \"status_description\" : \"{}\" }}",
            status.files[i].0, status.files[i].1, status.status[i].0, status.status[i].1,
        )?;
        if iterator.peek().is_some() {
            writeln!(writer, ",")?;
        } else {
            writeln!(writer)?;
        }
    }
    writeln!(writer, "]\n")?;
    writer.flush()?;
    Ok(())
}
