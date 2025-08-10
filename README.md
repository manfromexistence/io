# IO

https://github.com/astral-sh/uv good now learn from this uv github repo which is rust based python package manager and tell me what does they use for io operations and whatever they use create a rust code like this to show that methods time!!!

```bash
io-46530534:~/io{main}$ cargo run --release
   Compiling io v0.1.0 (/home/user/io)
    Finished `release` profile [optimized] target(s) in 0.54s
     Running `target/release/io`
Operation 'Create Directory' took: 0.10ms
10000 files created.
Operation 'Create 10000 files' took: 1121.51ms
10000 files updated.
Operation 'Update 10000 files' took: 972.60ms
Operation 'Read 10000 files' took: 178.61ms
Operation 'Delete 10000 files' took: 840.85ms
Operation 'Delete Directory' took: 18.25ms

Total time for all operations: 3135.92ms
io-46530534:~/io{main}$ cargo run --release
    Finished `release` profile [optimized] target(s) in 0.08s
     Running `target/release/io`
Operation 'Create Directory' took: 0.10ms
10000 files created.
Operation 'Create 10000 files' took: 692.16ms
10000 files updated.
Operation 'Update 10000 files' took: 1183.39ms
Operation 'Read 10000 files' took: 297.39ms
Operation 'Delete 10000 files' took: 634.74ms
Operation 'Delete Directory' took: 17.25ms

Total time for all operations: 2828.56ms
io-46530534:~/io{main}$ cargo run --release
    Finished `release` profile [optimized] target(s) in 0.13s
     Running `target/release/io`
Operation 'Create Directory' took: 0.10ms
10000 files created.
Operation 'Create 10000 files' took: 691.70ms
10000 files updated.
Operation 'Update 10000 files' took: 1053.71ms
Operation 'Read 10000 files' took: 175.44ms
Operation 'Delete 10000 files' took: 309.33ms
Operation 'Delete Directory' took: 6.09ms

Total time for all operations: 2236.53ms
io-46530534:~/io{main}$ cargo run --release
    Finished `release` profile [optimized] target(s) in 0.05s
     Running `target/release/io`
Operation 'Create Directory' took: 0.10ms
10000 files created.
Operation 'Create 10000 files' took: 976.79ms
10000 files updated.
Operation 'Update 10000 files' took: 1161.33ms
Operation 'Read 10000 files' took: 389.83ms
Operation 'Delete 10000 files' took: 775.69ms
Operation 'Delete Directory' took: 43.40ms

Total time for all operations: 3348.53ms
io-46530534:~/io{main}$ cargo run --release
    Finished `release` profile [optimized] target(s) in 0.13s
     Running `target/release/io`
Operation 'Create Directory' took: 0.37ms
10000 files created.
Operation 'Create 10000 files' took: 1146.97ms
10000 files updated.
Operation 'Update 10000 files' took: 1151.90ms
Operation 'Read 10000 files' took: 280.88ms
Operation 'Delete 10000 files' took: 434.22ms
Operation 'Delete Directory' took: 10.91ms
```


```md
In my rust code I have to ways to do io operations in one I showed a traditional io operation in rust vs my smart io and its 24% faster than the traditional one is it possible to make my smart io even faster list all of the suggestions about how much min faster it can get if I implement that with real data?

Performance
```bash
Overall Performance
Traditional I/O Average Time: 1044 ms

Smart I/O Average Time: 800 ms
```

Cargo.toml
```toml
[package]
name = "io"
version = "0.1.0"
edition = "2024"

[dependencies]
crossbeam-deque = "0.8.6"
futures = "0.3.31"
libc = "0.2.174"
memmap2 = "0.9.7"
rayon = "1.10.0"
tokio = { version = "1.47.1", features = ["rt-multi-thread", "fs", "io-util"] }
```

src/main.rs
```rust
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use libc::{sched_setaffinity, cpu_set_t, CPU_SET};
use memmap2::MmapMut;
use rayon::prelude::*;

const NUM_FILES: usize = 10000;
const CONTENT: &[u8] = b"initial content padded to simulate dx-check workload....................100 bytes..";
const UPDATE_CONTENT: &[u8] = b"updated content padded to simulate dx-check workload....................100 bytes..";

fn get_dir() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("bench_files");
    path
}

fn pin_thread(core_id: usize) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        unsafe {
            let mut cpu_set: cpu_set_t = std::mem::zeroed();
            CPU_SET(core_id, &mut cpu_set);
            let result = sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpu_set);
            if result != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}

fn run_in_pinned_pool<F>(benchmark_fn: F) -> io::Result<()>
where
    F: FnOnce() -> io::Result<()> + Send,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(rayon::current_num_threads())
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    pool.install(|| {
        (0..rayon::current_num_threads()).into_par_iter().for_each(|id| {
            let _ = pin_thread(id);
        });
        benchmark_fn()
    })
}

fn create_files(paths: &[PathBuf]) -> io::Result<()> {
    paths.par_iter().try_for_each(|path| {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(CONTENT)?;
        writer.flush()?;
        Ok::<(), io::Error>(())
    })
}

fn read_files(paths: &[PathBuf]) -> io::Result<()> {
    paths.par_iter().try_for_each(|path| {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok::<(), io::Error>(())
    })
}

fn update_files_traditionally(paths: &[PathBuf]) -> io::Result<()> {
    paths.par_iter().try_for_each(|path| {
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(UPDATE_CONTENT)?;
        writer.flush()?;
        Ok::<(), io::Error>(())
    })
}

fn update_files_smartly(paths: &[PathBuf]) -> io::Result<()> {
    paths.par_iter().try_for_each(|path| {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        if mmap.len() < UPDATE_CONTENT.len() {
            file.set_len(UPDATE_CONTENT.len() as u64)?;
            mmap = unsafe { MmapMut::map_mut(&file)? };
        }
        mmap[..UPDATE_CONTENT.len()].copy_from_slice(UPDATE_CONTENT);
        Ok::<(), io::Error>(())
    })
}

fn delete_files(paths: &[PathBuf]) -> io::Result<()> {
    paths.par_iter().try_for_each(fs::remove_file)
}

fn traditional_io() -> io::Result<()> {
    let dir_path = get_dir();
    let file_paths: Vec<_> = (0..NUM_FILES).map(|i| dir_path.join(format!("file_{}.txt", i))).collect();

    let start = Instant::now();
    create_files(&file_paths)?;
    let create_time = start.elapsed().as_millis();

    let start = Instant::now();
    read_files(&file_paths)?;
    let read_time = start.elapsed().as_millis();

    let start = Instant::now();
    update_files_traditionally(&file_paths)?;
    let update_time = start.elapsed().as_millis();

    let start = Instant::now();
    delete_files(&file_paths)?;
    let delete_time = start.elapsed().as_millis();

    println!("Traditional times (ms): Create: {}, Read: {}, Update: {}, Delete: {}", create_time, read_time, update_time, delete_time);
    println!("Total: {} ms", create_time + read_time + update_time + delete_time);
    Ok(())
}

fn smart_io() -> io::Result<()> {
    let dir_path = get_dir();
    let file_paths: Vec<_> = (0..NUM_FILES).map(|i| dir_path.join(format!("file_{}.txt", i))).collect();

    let start = Instant::now();
    create_files(&file_paths)?;
    let create_time = start.elapsed().as_millis();

    let start = Instant::now();
    read_files(&file_paths)?;
    let read_time = start.elapsed().as_millis();

    let start = Instant::now();
    update_files_smartly(&file_paths)?;
    let update_time = start.elapsed().as_millis();

    let start = Instant::now();
    delete_files(&file_paths)?;
    let delete_time = start.elapsed().as_millis();

    println!("Smart times (ms): Create: {}, Read: {}, Update: {}, Delete: {}", create_time, read_time, update_time, delete_time);
    println!("Total: {} ms", create_time + read_time + update_time + delete_time);
    Ok(())
}

fn main() -> io::Result<()> {
    let dir_path = get_dir();
    fs::create_dir_all(&dir_path)?;

    println!("Running traditional_io...");
    run_in_pinned_pool(traditional_io)?;

    println!("\nRunning smart_io (mmap Update Only)...");
    run_in_pinned_pool(smart_io)?;

    fs::remove_dir_all(&dir_path)?;
    Ok(())
}
```
```

```bash
cargo add tokio-uring memmap2 rayon tokio crossbeam-deque libc
```