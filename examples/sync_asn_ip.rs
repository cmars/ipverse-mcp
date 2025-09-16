use ipverse_mcp::asn_ip::upstream::Upstream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let upstream = Upstream::new().expect("create upstream");

    // Initial setup
    upstream.provision().expect("provision");

    // Update and get changed files
    let changed_files = upstream.update().expect("update");
    println!(
        "Changed files: {}",
        changed_files
            .iter()
            .map(|path| path.to_str())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect::<Vec<&str>>()
            .join(", ".into())
    );

    // Get path to a specific ASN's data
    let asn_file = upstream.get_asn_file_path(15169); // Example ASN (Google)
    println!("ASN 15169 data file path: {:?}", asn_file);

    Ok(())
}
