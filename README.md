# gb-rs

Personal Game Boy emulator made in Rust.

## Usage

```
cargo run <path to ROM>
```

## Controls

* Enter: START
* Space: SELECT
* Arrow Keys: D-pad
* A (or Q in AZERTY): A
* S: B

## TODO

- [ ] Audio
- [ ] More Memory Bank Controllers
- [ ] Switch to better GUI library

## Bibliography

http://www.codeslinger.co.uk/pages/projects/gameboy/beginning.html

Ultimate Game Boy Talk: https://www.youtube.com/watch?v=HyzD8pNlpwI

### CPU

Detailled instructions: https://rgbds.gbdev.io/docs/v0.5.2/gbz80.7#LD__HLI_,A

Opcode table: https://izik1.github.io/gbops/index.html

### Memory bank controllers

https://b13rg.github.io/Gameboy-MBC-Analysis/

https://b13rg.github.io/Gameboy-Bank-Switching/

gbdev: https://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers#MBC3_.28max_2MByte_ROM_and.2For_64KByte_RAM_and_Timer.29

### Test ROMs

https://github.com/mattcurrie/dmg-acid2

https://github.com/retrio/gb-test-roms
