# RustyCalicoC8

Cross-platform Chip8 emulator written in Rust. Port of my existing [CalicoC8](https://github.com/Xiperiz/calico-c8).

### Progress

- [x] Graphics
- [x] Sound
- [x] Input
- [x] All instructions
- [x] User configurable window size
- [x] User configurable clock speed

## Getting Started

### Dependencies

* Rust Toolchain with Cargo support.
* [SDL2]("https://www.libsdl.org")

### Usage

* Clone the repository

```
git clone https://github.com/Xiperiz/rusty-calico-c8
```

* Compile the program

```
cargo build --release
```

* Launch a ROM

```
rusty-calico-c8 <path to rom or 'help'> <args>
```

### Command line arguments

* -no_sound - disables 'beep' sound.
* -clock_speed:x - sets clock speed to X hz
* -window_size:x:y - sets window size to X by Y

The arguments with values need to have a format specified above (-arg:val), below is an example with all of the
arguments used together:

```
rusty-calico-c8 SpaceInvaders.ch8 -no_sound -clock_speed:800 -window_size:1280:640
```

You can omit any argument and the default will be used, below are default values for each argument:

* -no_sound - false
* -clock_speed - 600hz
* -window_size - 640 x 320

Keep in mind there are no checks for the values, if you put ridiculous values then expect unexpected behaviour!

### Input

Following CHIP8 keypad

| 1 | 2 | 3 | C |
|---|---|---|---|
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

was mapped to keys

| 1 | 2 | 3 | 4 |
|---|---|---|---|
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

## License

This project is licensed under the [GNU AGPLv3] License - see the [LICENSE.md](LICENSE.md) file for details.
