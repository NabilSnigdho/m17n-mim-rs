# m17n-mim-rs

Parser and converter for the M17N .mim input method format.

## Implementation status

- [x] Basic insert, shift actions support
- [x] delete, move, set, commit, cond actions support
- [x] Surrounding text support
- [ ] Global variables and commands
- [ ] Custom commands, macros support
- [x] Convert input string
- [ ] Full fledged IME

## Usage

For Rust:

```rust
use m17n_mim_rs::M17nMim;

let mim_str = include_str!("path/to/your.mim");
let mim = M17nMim::new(mim_str);

let input = "your input string";
let output = mim.convert(input);
```

For JS:

```js
import { M17nMim } from "m17n-mim-wasm";

const mimStr = "your mim string";
const mim = new M17nMim(mimStr);

const input = "your input string";
const output = mim.convert(input);
```

## License
This project includes portions derived from the [M17N library](https://www.nongnu.org/m17n/),
Copyright © 2003–2012
National Institute of Advanced Industrial Science and Technology (AIST),
distributed under the GNU Lesser General Public License v2.1 or later.

All modifications and original work © 2025 Mahmud Nabil,
licensed under the same terms (LGPL-2.1-or-later).
