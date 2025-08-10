use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let num_files = 10_000;
    let dir = "modules/temp_files";
    fs::create_dir_all(dir)?;

    // Generate file paths
    let file_paths: Vec<String> = (0..num_files)
        .map(|i| format!("{}/file_{}.txt", dir, i))
        .collect();

    let total_start = Instant::now();

    // Create files
    let create_start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::create(path)?;
        write!(file, "Initial content")?;
        file.flush()?;
        Ok::<(), std::io::Error>(())
    })?;
    let create_duration = create_start.elapsed();
    println!("Create time: {} ms", create_duration.as_millis());

    // Read files
    let read_start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok::<(), std::io::Error>(())
    })?;
    let read_duration = read_start.elapsed();
    println!("Read time: {} ms", read_duration.as_millis());

    // Update files
    let update_start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::create(path)?;
        write!(file, "Updated content")?;
        file.flush()?;
        Ok::<(), std::io::Error>(())
    })?;
    let update_duration = update_start.elapsed();
    println!("Update time: {} ms", update_duration.as_millis());

    // Delete files
    let delete_start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        fs::remove_file(path)?;
        Ok::<(), std::io::Error>(())
    })?;
    let delete_duration = delete_start.elapsed();
    println!("Delete time: {} ms", delete_duration.as_millis());

    // Clean up directory
    fs::remove_dir_all("modules")?;

    let total_duration = total_start.elapsed();
    println!("Total time taken: {} ms", total_duration.as_millis());

    Ok(())
}