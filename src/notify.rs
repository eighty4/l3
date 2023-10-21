use std::path::Path;
use notify::{Watcher, RecursiveMode, Result};

fn main() -> Result<()> {
    let mut watcher = notify::recommended_watcher(|res| {
        match res {
            Ok(event) => println!("event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    })?;
    watcher.watch(Path::new("foo"), RecursiveMode::Recursive)?;

    std::thread::sleep(std::time::Duration::from_secs(60));

    Ok(())
}
