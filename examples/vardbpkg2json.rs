

// Example tool that parses the vardb and outputs it as JSON.
// Usage: cargo run --example vardbpkg2json -- [path]
// Default path is /var/db/pkg if no path is provided.

use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "/var/db/pkg"
    };

    println!("Scanning directory: {}", path);
    let packages = vardbpkg::parse_vardb(Path::new(path));
    
    let json = serde_json::to_string_pretty(&packages)?;
    println!("{}", json);

    Ok(())
}
