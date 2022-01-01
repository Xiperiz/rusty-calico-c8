use std::process::exit;

use crate::cmd_args::ApplicationCmdSettings;
use crate::emulator::Emulator;

mod cmd_args;
mod emulator;
mod interpreter;
mod frame_buffer;
mod audio;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 || args[1] == "help" {
        println!("usage: rusty-calico-c8 <rom-path or 'help> <args>");
        println!("args explanation:");
        println!("-window_size:x:y = sets window width to 'x' and height to 'y' (default = 640 x 320)");
        println!("-clock_speed:x = sets clock speed to 'x' (default = 600)");
        println!("-no_sound = disables the beep sound (default = false)");

        return;
    }

    let rom_path = &args[1];

    let parsed_args = if args.len() == 2 {
        ApplicationCmdSettings::new()
    } else {
        match ApplicationCmdSettings::new_from_args(&args) {
            Ok(val) => val,
            Err(e) => {
                println!("{}", e);

                exit(-1)
            }
        }
    };

    match Emulator::new(parsed_args).run(rom_path) {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);

            exit(-1)
        }
    }
}
