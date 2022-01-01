use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::cmd_args::CommandLineArgError::{InvalidArgument, InvalidArgumentOptionCount, InvalidArgumentOptionParse};

#[derive(Debug, PartialEq)]
pub enum CommandLineArgError<'a> {
    InvalidArgument { arg: &'a String },
    InvalidArgumentOptionCount { arg: &'a String },
    InvalidArgumentOptionParse { arg: &'a String, value: &'a str },
}

impl Display for CommandLineArgError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            CommandLineArgError::InvalidArgument { arg } => {
                write!(f, "Invalid argument '{}'", arg)
            }

            CommandLineArgError::InvalidArgumentOptionCount { arg } => {
                write!(f, "Invalid argument '{}' format", arg)
            }

            CommandLineArgError::InvalidArgumentOptionParse { arg, value } => {
                write!(f, "Unable to parse argument's '{0}' option '{1}'", arg, value)
            }
        }
    }
}

impl Error for CommandLineArgError<'_> {}

#[derive(Debug, PartialEq)]
pub struct ApplicationCmdSettings {
    pub sound_enabled: bool,
    pub window_size_x: u32,
    pub window_size_y: u32,
    pub cpu_clock_speed: u64,
}

impl ApplicationCmdSettings {
    pub fn new() -> ApplicationCmdSettings {
        ApplicationCmdSettings {
            sound_enabled: true,
            window_size_x: 640,
            window_size_y: 320,
            cpu_clock_speed: 600,
        }
    }

    pub fn new_from_args(args: &Vec<String>) -> Result<ApplicationCmdSettings, CommandLineArgError> {
        let mut res = ApplicationCmdSettings::new();

        for (i, arg) in args.iter().enumerate() {
            if i == 0 || i == 1 {
                continue;
            }

            let arg_tokens: Vec<_> = arg.split(":").collect();

            match arg_tokens[0] {
                "-no_sound" => {
                    if arg_tokens.len() != 1 {
                        return Err(InvalidArgumentOptionCount { arg });
                    }

                    res.sound_enabled = false;
                }

                "-clock_speed" => {
                    if arg_tokens.len() != 2 {
                        return Err(InvalidArgumentOptionCount { arg });
                    }

                    match arg_tokens[1].parse() {
                        Ok(val) => res.cpu_clock_speed = val,
                        Err(_) => return Err(InvalidArgumentOptionParse { arg, value: arg_tokens[1] })
                    }
                }

                "-window_size" => {
                    if arg_tokens.len() != 3 {
                        return Err(InvalidArgumentOptionCount { arg });
                    }

                    match arg_tokens[1].parse() {
                        Ok(val) => res.window_size_x = val,
                        Err(_) => return Err(InvalidArgumentOptionParse { arg, value: arg_tokens[1] })
                    }

                    match arg_tokens[2].parse() {
                        Ok(val) => res.window_size_y = val,
                        Err(_) => return Err(InvalidArgumentOptionParse { arg, value: arg_tokens[2] })
                    }
                }

                _ => return Err(InvalidArgument { arg })
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_from_args_valid_test() {
        let args: Vec<String> = vec!["rusty-calico-c8".to_owned(), "rom.ch8".to_owned(), "-no_sound".to_owned(), "-clock_speed:780".to_owned(),
                                     "-window_size:1280:640".to_owned()];

        let res = ApplicationCmdSettings::new_from_args(&args).unwrap();

        assert_eq!(res.cpu_clock_speed, 780);
        assert_eq!(res.sound_enabled, false);
        assert_eq!(res.window_size_x, 1280);
        assert_eq!(res.window_size_y, 640);
    }

    #[test]
    fn new_from_args_invalid_test() {
        let mut args: Vec<String> = vec!["rusty-calico-c8".to_owned(), "rom.ch8".to_owned(), "-sound".to_owned(), "-clock_speed:780".to_owned(),
                                         "-window_size:1280:640".to_owned()];

        let mut res = ApplicationCmdSettings::new_from_args(&args);

        assert_eq!(res, Err(CommandLineArgError::InvalidArgument { arg: &"-sound".to_owned() }));

        args = vec!["rusty-calico-c8".to_owned(), "rom.ch8".to_owned(), "-no_sound".to_owned(), "-clock_speed:780".to_owned(),
                    "-window_size:12o0:640".to_owned()];

        res = ApplicationCmdSettings::new_from_args(&args);

        assert_eq!(res, Err(CommandLineArgError::InvalidArgumentOptionParse {
            arg: &"-window_size:12o0:640".to_owned(),
            value: "12o0",
        }));

        args = vec!["rusty-calico-c8".to_owned(), "rom.ch8".to_owned(), "-no_sound".to_owned(), "-clock_speed:780:12".to_owned(),
                    "-window_size:1280:640".to_owned()];

        res = ApplicationCmdSettings::new_from_args(&args);

        assert_eq!(res, Err(CommandLineArgError::InvalidArgumentOptionCount { arg: &"-clock_speed:780:12".to_owned() }));
    }
}
