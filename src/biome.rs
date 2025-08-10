use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let num_files = 10_000;
    let dir = "temp_files";
    fs::create_dir_all(dir)?;

    // Generate file paths
    let file_paths: Vec<String> = (0..num_files)
        .map(|i| format!("{}/file_{}.txt", dir, i))
        .collect();

    let start = Instant::now();

    // Create files
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::create(path)?;
        write!(file, "Initial content")?;
        file.flush()?;
        Ok::<(), std::io::Error>(())
    })?;

    // Read files
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok::<(), std::io::Error>(())
    })?;

    // Update files
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::create(path)?;
        write!(file, "Updated content")?;
        file.flush()?;
        Ok::<(), std::io::Error>(())
    })?;

    // Delete files
    file_paths.par_iter().try_for_each(|path| {
        fs::remove_file(path)?;
        Ok::<(), std::io::Error>(())
    })?;

    // Clean up directory
    fs::remove_dir(dir)?;

    let duration = start.elapsed();
    println!("Total time taken: {:.3} seconds", duration.as_secs_f64());

    Ok(())
}