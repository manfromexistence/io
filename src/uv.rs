use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

const NUM_FILES: usize = 10000;

mod file_operations {
    use super::*;

    pub fn create_and_write_file(path: &Path, content: &str) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn read_file(path: &Path) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}

mod directory_operations {
    use super::*;

    pub fn create_directory(path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(path)
    }
}

fn time_operation<F, T>(operation_name: &str, operation: F) -> T
where
    F: FnOnce() -> T,
{
    let start_time = Instant::now();
    let result = operation();
    let elapsed_time = start_time.elapsed();
    println!(
        "Operation '{}' took: {:.2}ms",
        operation_name,
        elapsed_time.as_secs_f64() * 1000.0
    );
    result
}

fn main() {
    let total_start_time = Instant::now();
    let dir_path = Path::new("modules");

    time_operation("Create Directory", || {
        if let Err(e) = directory_operations::create_directory(dir_path) {
            eprintln!("Failed to create directory: {}", e);
            return;
        }
    });

    time_operation(&format!("Create {} files", NUM_FILES), || {
        for i in 0..NUM_FILES {
            let file_path = dir_path.join(format!("file_{}.txt", i));
            let content = format!("Hello from file {}!", i);
            if let Err(e) = file_operations::create_and_write_file(&file_path, &content) {
                eprintln!("Failed to create file {}: {}", i, e);
            }
        }
        println!("{} files created.", NUM_FILES);
    });

    time_operation(&format!("Update {} files", NUM_FILES), || {
        for i in 0..NUM_FILES {
            let file_path = dir_path.join(format!("file_{}.txt", i));
            let content = format!("This is updated content for file {}!", i);
            if let Err(e) = file_operations::create_and_write_file(&file_path, &content) {
                eprintln!("Failed to update file {}: {}", i, e);
            }
        }
        println!("{} files updated.", NUM_FILES);
    });

    time_operation(&format!("Read {} files", NUM_FILES), || {
        for i in 0..NUM_FILES {
            let file_path = dir_path.join(format!("file_{}.txt", i));
            if let Err(e) = file_operations::read_file(&file_path) {
                eprintln!("Failed to read file {}: {}", i, e);
            }
        }
    });

    time_operation(&format!("Delete {} files", NUM_FILES), || {
        for i in 0..NUM_FILES {
            let file_path = dir_path.join(format!("file_{}.txt", i));
            if let Err(e) = fs::remove_file(&file_path) {
                eprintln!("Failed to delete file {}: {}", i, e);
            }
        }
    });

    time_operation("Delete Directory", || {
        if let Err(e) = fs::remove_dir(dir_path) {
            eprintln!("Failed to delete directory: {}", e);
        }
    });

    let total_elapsed_time = total_start_time.elapsed();
    println!(
        "\nTotal time for all operations: {:.2}ms",
        total_elapsed_time.as_secs_f64() * 1000.0
    );
}
