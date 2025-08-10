use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write, Read};
use std::time::Instant;
use rayon::prelude::*;
// Import MmapMut for the smart update. Mmap (read-only) is no longer needed here.
use memmap2::MmapMut;
use libc::{sched_setaffinity, cpu_set_t};

// Tokio and futures are no longer needed for the hybrid smart_io function.

const NUM_FILES: usize = 10000;
// Using env::temp_dir() is a robust way to get a temporary directory.
fn get_dir() -> std::path::PathBuf {
    let mut path = env::temp_dir();
    path.push("bench_files");
    path
}
const CONTENT: &[u8] = b"initial content padded to simulate dx-check workload....................100 bytes..";
const UPDATE_CONTENT: &[u8] = b"updated content padded to simulate dx-check workload....................100 bytes..";

fn main() -> io::Result<()> {
    let dir_path = get_dir();
    fs::create_dir_all(&dir_path)?;

    println!("Running traditional_io...");
    traditional_io()?;

    println!("\nRunning smart_io (hybrid)...");
    smart_io()?;

    // Cleanup (optional, comment out if testing)
    fs::remove_dir_all(&dir_path)?;
    Ok(())
}

// Traditional: Basic Rayon parallelism + std::fs with BufWriter. This function is unchanged.
fn traditional_io() -> io::Result<()> {
    let dir_path = get_dir();
    let file_paths: Vec<_> = (0..NUM_FILES).map(|i| dir_path.join(format!("file_{}.txt", i))).collect();

    // Create
    let start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(CONTENT)?;
        writer.flush()?;
        Ok::<(), io::Error>(())
    })?;
    let create_time = start.elapsed().as_millis();

    // Read
    let start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok::<(), io::Error>(())
    })?;
    let read_time = start.elapsed().as_millis();

    // Update (rewrite content)
    let start = Instant::now();
    file_paths.par_iter().try_for_each(|path| {
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(UPDATE_CONTENT)?;
        writer.flush()?;
        Ok::<(), io::Error>(())
    })?;
    let update_time = start.elapsed().as_millis();

    // Delete
    let start = Instant::now();
    file_paths.par_iter().try_for_each(|path| fs::remove_file(path))?;
    let delete_time = start.elapsed().as_millis();

    println!("Traditional times (ms): Create: {}, Read: {}, Update: {}, Delete: {}", create_time, read_time, update_time, delete_time);
    println!("Total: {} ms", create_time + read_time + update_time + delete_time);
    Ok(())
}

// Smart (Hybrid): Uses traditional I/O for C/R/D, but mmap for Update.
fn smart_io() -> io::Result<()> {
    let dir_path = get_dir();
    let file_paths: Vec<_> = (0..NUM_FILES).map(|i| dir_path.join(format!("file_{}.txt", i))).collect();

    // Enhanced Rayon init with thread pinning
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(rayon::current_num_threads())
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    pool.install(|| {
        // Pin threads, but handle potential errors gracefully
        let pin_result = (0..rayon::current_num_threads()).into_par_iter().try_for_each(|id| pin_thread(id));
        if let Err(e) = pin_result {
             eprintln!("Warning: Could not pin threads to cores: {}. This can happen in some environments (like containers). Continuing without pinning.", e);
        }

        // Create (using traditional method)
        let start = Instant::now();
        file_paths.par_iter().try_for_each(|path| {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(CONTENT)?;
            writer.flush()?;
            Ok::<(), io::Error>(())
        })?;
        let create_time = start.elapsed().as_millis();

        // Read (using traditional method)
        let start = Instant::now();
        file_paths.par_iter().try_for_each(|path| {
            let mut file = File::open(path)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            Ok::<(), io::Error>(())
        })?;
        let read_time = start.elapsed().as_millis();

        // Update with mmap (the "smart" part)
        let start = Instant::now();
        file_paths.par_iter().try_for_each(|path| {
            let file = OpenOptions::new().read(true).write(true).open(path)?;
            let mut mmap = unsafe { MmapMut::map_mut(&file)? };
            if mmap.len() < UPDATE_CONTENT.len() {
                file.set_len(UPDATE_CONTENT.len() as u64)?;
                mmap = unsafe { MmapMut::map_mut(&file)? };
            }
            mmap[..UPDATE_CONTENT.len()].copy_from_slice(UPDATE_CONTENT);
            // No flush() needed, OS handles it efficiently.
            Ok::<(), io::Error>(())
        })?;
        let update_time = start.elapsed().as_millis();

        // Delete (using traditional method)
        let start = Instant::now();
        file_paths.par_iter().try_for_each(|path| fs::remove_file(path))?;
        let delete_time = start.elapsed().as_millis();

        println!("Smart times (ms): Create: {}, Read: {}, Update: {}, Delete: {}", create_time, read_time, update_time, delete_time);
        println!("Total: {} ms", create_time + read_time + update_time + delete_time);
        Ok(())
    })
}

// Modified to return a Result to handle potential errors.
fn pin_thread(core_id: usize) -> io::Result<()> {
    // This function is platform-specific and might not work on all OSes or environments.
    #[cfg(target_os = "linux")]
    {
        unsafe {
            let mut cpu_set: cpu_set_t = std::mem::zeroed();
            libc::CPU_SET(core_id, &mut cpu_set);
            let result = sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpu_set);
            if result != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}
