# chip-8-rs

👾 A portable [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator written in Rust, and powered by [SDL2](https://www.libsdl.org/)

## Building

Clone the repository and run `cargo build` to compile the app.

## Controls

The COSMAC VIP keypad layout is mapped to the left side of the QWERTY keyboard:

<table>
<tr><th>COSMAC VIP keypad</th><th>QWERTY layout</th></tr>
<tr>
  <td>
  
| 1 | 2 | 3 | C |
|:-:|:-:|:-:|:-:|
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |
  
  </td>
  <td>

| 1 | 2 | 3 | 4 |
|:-:|:-:|:-:|:-:|
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

  </td>
</tr>
</table>

There are also a bunch of control commands:

- **Ctrl+O:** Open file
- **Ctrl+Q:** Quit
- **Ctrl+R:** Reset program
- **Ctrl+C:** Change pixel color
- **Ctrl+W:** Decrease CPU execution speed
- **Ctrl+E:** Increase CPU execution speed
