extern crate libc;

use libltc_rs::{
    consts::{LtcBgFlags, LtcBgFlagsKind},
    encoder::LTCEncoder,
    LTCTVStandard, SMPTETimecode, Timezone,
};
use std::io::Write;

use std::env;
use std::fs::File;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename;
    let mut sample_rate = 48000.0;
    let mut fps = 25.0;
    let mut length = 2.0;

    if args.len() > 1 {
        filename = &args[1];
        if args.len() > 2 {
            sample_rate = args[2].parse().unwrap_or(48000.0);
        }
        if args.len() > 3 {
            fps = args[3].parse().unwrap_or(25.0);
        }
        if args.len() > 4 {
            length = args[4].parse().unwrap_or(2.0);
        }
    } else {
        eprintln!("ltcencode - test/example application to encode LTC to a file\n");
        eprintln!("Usage: ltcencode <filename> [sample rate [frame rate [duration in s]]]\n");
        eprintln!("default-values:");
        eprintln!(" sample rate: 48000.0 [SPS], frame rate: 25.0 [fps], duration: 2.0 [sec]\n");
        eprintln!("Report bugs to Robin Gareus <robin@gareus.org>\n");
        exit(1);
    }

    let file = match File::create(filename) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Error: cannot open file '{}' for writing.", filename);
            exit(1);
        }
    };

    // Initialize the timecode structure
    let timezone: Timezone = b"+00100".into();
    let st = SMPTETimecode::new(timezone, 3, 1, 10, 0, 0, 0, 1);
    let flags = *LtcBgFlags::default().set(LtcBgFlagsKind::LTC_USE_DATE);

    // Initialize the LTC Encoder
    let mut encoder = LTCEncoder::try_new(1.0, 1.0, LTCTVStandard::default(), flags).unwrap();

    encoder.set_buffersize(sample_rate, fps).unwrap();
    encoder
        .reinit(
            sample_rate,
            fps,
            if fps == 25.0 {
                LTCTVStandard::LTCTV_625_50
            } else {
                LTCTVStandard::LTCTV_525_60
            },
            flags,
        )
        .unwrap();

    encoder.set_filter(0.0);
    encoder.set_filter(25.0);
    encoder.set_volume(-18.0).unwrap();

    encoder.set_timecode(&st);

    println!("sample rate: {:.2}", sample_rate);
    println!("frames/sec: {:.2}", fps);
    println!("secs to write: {:.2}", length);
    println!("sample format: 8bit unsigned mono");

    let vframe_last = (length * fps) as i32;
    let mut total_samples = 0;
    let mut file = file;

    for _ in 0..vframe_last {
        encoder.encode_frame();

        let (buf, len) = encoder.get_buf_ref(true);

        // In the loop where you process frames
        if len > 0 {
            // Assuming buf is a slice of raw bytes or samples, you need to write this to the file
            match file.write_all(&buf[..len]) {
                Ok(_) => total_samples += len as usize, // Increment the total samples written
                Err(e) => {
                    eprintln!("Error writing to file: {}", e);
                    exit(1);
                }
            }
        }
        encoder.inc_timecode().unwrap();
    }

    println!("Done: wrote {} samples to '{}'", total_samples, filename);
}