# Game of Life

A terminal-based implementation of Conway's Game of Life built with Rust and [ratatui](https://ratatui.rs/).

## Installation

```bash
cargo install --git <your-repo-url>
```

## Usage

Generate a random 20x20 board:
```bash
life
```

Load a pattern from a file:
```bash
life --from-file pattern.life
```

Customize board size and alive probability:
```bash
life --width 40 --height 30 --alive-probability 0.3
```

Set initial speed:
```bash
life --speed fast  # Options: slow, normal, fast
```

## Controls

- `q` or `Esc` - Quit
- `Space` - Pause/Unpause
- `↑` or `→` - Speed up
- `↓` or `←` - Slow down

## File Format

Pattern files use `.life` extension with simple text format:
- `X` = alive cell
- `O` = dead cell
- All rows must be the same length

Example (`glider.life`):
```
OOOOO
OOXOO
OOOXO
OXXXO
OOOOO
```

## License

MIT
