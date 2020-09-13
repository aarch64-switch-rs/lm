use std::fs;
use std::env;
use std::io::Read;
use logpacket::LogPacket;

fn main() {
    // TODO: use a better way to parse args, like clap
    let mut args: Vec<String> = env::args().skip(1).collect();
    let search_dir = match args.is_empty() {
        true => String::from(env::current_dir().unwrap().to_str().unwrap()),
        false => args[0].clone()
    };

    let mut verbose = false;
    if args.len() > 1 {
        args.drain(0..1);
        for arg in args {
            match arg.as_str() {
                "--verbose" | "-v" => verbose = true,
                _ => {}
            };
        }
    }
    
    let mut log_packet_table: Vec<(u64, LogPacket)> = Vec::new();
    for entry_v in fs::read_dir(search_dir).unwrap() {
        let mut log_packet_ok = false;
        if let Ok(entry) = entry_v {
            let file_path = entry.path();
            if let Ok(tick) = u64::from_str_radix(entry.file_name().to_string_lossy().trim_start_matches("0x").trim_end_matches(".nxbinlog"), 16) {
                if let Ok(mut file) = fs::File::open(&file_path) {
                    if let Ok(file_metadata) = fs::metadata(&file_path) {
                        let mut data: Vec<u8> = vec![0; file_metadata.len() as usize];
                        if let Ok(_) = file.read(&mut data) {
                            if let Some(log_packet) = LogPacket::from(data) {
                                log_packet_ok = true;
                                if !log_packet.is_head() {
                                    if let Some(ref mut last_log_packet) = log_packet_table.last_mut() {
                                        last_log_packet.1.try_join(log_packet);
                                        continue;
                                    }
                                }
                                log_packet_table.push((tick, log_packet));
                            }
                        }
                    }
                }
            }
        }
        if !log_packet_ok {
            println!("Unable to load log packet!");
        }
    }

    if !log_packet_table.is_empty() {
        log_packet_table.sort_by(|a, b| a.0.cmp(&b.0));
    }
    
    for log_packet in log_packet_table {
        if verbose {
            todo!();
        }
        else {
            // Only print text log.
            if let Some(text_log) = log_packet.1.get_text_log() {
                print!("{}", text_log);
            }
        }
    }
}
