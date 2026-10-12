//! Part 5 colour-accuracy harness, owner side (P1-clean).
//!
//! Compares two screenshots (ours vs a private reference capture) and
//! prints NUMBERS ONLY — per-cell mean RGB for both sides plus delta
//! stats. The owner runs this on their machine against captures that
//! must never enter the repo, and pastes the numbers back; tuning
//! follows from diffs, never from pixels in-tree.
//!
//! Usage: colordiff <ours.png> <theirs.png> [grid=8]
//! Exit nonzero on unreadable input; size mismatch compares the
//! overlap region (reported) instead of failing.

fn load_rgba(path: &str) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::open(path).ok()?.to_rgba8();
    let (w, h) = (img.width(), img.height());
    Some((w, h, img.into_raw()))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let fail = |msg: &str| -> ! {
        eprintln!("colordiff: {msg}");
        std::process::exit(2);
    };
    if args.len() < 3 {
        fail("usage: colordiff <ours.png> <theirs.png> [grid=8]");
    }
    let grid: u32 = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
        .clamp(1, 32);
    let Some((w1, h1, a)) = load_rgba(&args[1]) else {
        fail(&format!("cannot read {}", args[1]));
    };
    let Some((w2, h2, b)) = load_rgba(&args[2]) else {
        fail(&format!("cannot read {}", args[2]));
    };
    let (w, h) = (w1.min(w2), h1.min(h2));
    if w == 0 || h == 0 {
        fail("empty overlap region");
    }
    println!("colordiff: ours={w1}x{h1} theirs={w2}x{h2} overlap={w}x{h} grid={grid}");
    let mut hot = 0u32;
    for gy in 0..grid {
        for gx in 0..grid {
            let (x0, y0) = (gx * w / grid, gy * h / grid);
            let (x1, y1) = ((gx + 1) * w / grid, (gy + 1) * h / grid);
            let (mut n, mut sa, mut sb, mut maxd) = (0u64, [0u64; 3], [0u64; 3], 0u64);
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * w1 + x) * 4) as usize;
                    let j = ((y * w2 + x) * 4) as usize;
                    let (pa, pb) = (&a[i..i + 3], &b[j..j + 3]);
                    let mut d = 0u64;
                    for c in 0..3 {
                        sa[c] += pa[c] as u64;
                        sb[c] += pb[c] as u64;
                        d += (pa[c] as i32 - pb[c] as i32).unsigned_abs() as u64;
                    }
                    maxd = maxd.max(d / 3);
                    n += 1;
                }
            }
            if n == 0 {
                continue;
            }
            let ma = [sa[0] / n, sa[1] / n, sa[2] / n];
            let mb = [sb[0] / n, sb[1] / n, sb[2] / n];
            let md = ((ma[0] as i64 - mb[0] as i64).abs()
                + (ma[1] as i64 - mb[1] as i64).abs()
                + (ma[2] as i64 - mb[2] as i64).abs()) as u64
                / 3;
            if md > 8 {
                hot += 1;
            }
            println!(
                "cell {gx},{gy}: ours={:?} theirs={:?} mean-delta={md} max-px-delta={maxd}",
                [ma[0], ma[1], ma[2]],
                [mb[0], mb[1], mb[2]]
            );
        }
    }
    println!("colordiff: cells-over-8 = {hot} of {}", grid * grid);
}
