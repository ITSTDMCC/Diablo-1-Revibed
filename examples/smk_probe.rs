//! Developer probe: decode a Smacker movie from an MPQ, print its header and write some frames
//! as PNG (game data: write them outside the repository).
//! cargo run --release --example smk_probe -- <path.mpq> <movie> <out_dir> [frame,frame,...]
use diablo1_rs::mpq::MpqArchive;
use diablo1_rs::storm::smacker::Smacker;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut mpq = MpqArchive::open(&args[1]).unwrap().expect("archive exists");
    let data = mpq.read_file(&args[2].replace('/', "\\")).expect("movie in archive");
    let out = std::path::Path::new(&args[3]);
    let want: Vec<u32> = args.get(4).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect()).unwrap_or_default();
    let mut smk = Smacker::open(data).expect("valid Smacker file");
    let (w, h) = smk.frame_size();
    println!("{}x{} frames={} fps={} audio0={:?}", w, h, smk.num_frames(), smk.frame_rate(), smk.audio_track_details(0));
    let mut buf = vec![0u8; (w * h) as usize];
    let mut audio_bytes = 0usize;
    let t = std::time::Instant::now();
    for f in 0..smk.num_frames() {
        smk.get_next_frame().expect("frame decodes");
        audio_bytes += smk.audio_data(0).len();
        if want.contains(&f) {
            smk.get_frame(&mut buf);
            let pal = smk.palette();
            let mut rgba = Vec::with_capacity(buf.len() * 4);
            for &p in &buf {
                rgba.extend_from_slice(&pal[p as usize * 3..p as usize * 3 + 3]);
                rgba.push(255);
            }
            diablo1_rs::platform::png::write_rgba(&out.join(format!("frame_{f}.png")), w, h, &rgba).unwrap();
        }
    }
    println!("decoded in {:?}, audio bytes {}", t.elapsed(), audio_bytes);
}
