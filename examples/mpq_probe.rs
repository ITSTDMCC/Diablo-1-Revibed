//! Developer probe: list block-table flag usage and read the listfile of an MPQ.
//! cargo run --example mpq_probe -- <path.mpq> [file-in-archive]
use diablo1_rs::mpq::MpqArchive;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut mpq = MpqArchive::open(&args[1]).unwrap().expect("archive exists");
    let name = args.get(2).map(String::as_str).unwrap_or("(listfile)");
    match mpq.read_file(name) {
        Ok(b) => println!("{name}: {} bytes\n{}", b.len(), String::from_utf8_lossy(&b[..b.len().min(400)])),
        Err(e) => println!("{name}: {e}"),
    }
}
