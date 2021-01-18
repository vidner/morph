# Morph
Polymorphic ELF Runtime Crypter written in rust.

## How to use
Either you build the binary or run straight with cargo.

```
./morph program process 
```
```
cargo run program process
```
The executable will be at `<program>_morph` directory.

## Demo
Polymorhpic `self-modifying`.

![](https://i.imgur.com/jrlwe22.png)

Runtime `run-from-memory`.

![](https://i.imgur.com/pdWfdPe.png)

## Todos
 - better encryption.
 - better anti-debug.
 - remove panic string.
 - unpacker.
 

## References
[linux-elf-runtime-crypter](https://www.guitmz.com/linux-elf-runtime-crypter/)
