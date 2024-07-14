# glitch-rust

Implementation of [glitch](https://github.com/jpoz/glitch/tree/master?tab=readme-ov-file) in rust (originally in Go). Didn't implement all, just some that interested me.

Just for fun and playing around with image transformations.

## Usage
1. `git clone https://github.com/willxxy/glitch-rust.git`
2. `cd glitch-rust`
3. `cargo build`
4. `cargo run -- --file input.png --effect $EFFECT` where EFFECT={copy, transpose_input, vertical_transpose_input, channel_shift_left, channel_shift_right, half_life_right, prism_burst, noise, compression_ghost, all}.

## Examples

### copy:
![copy](./pngs/output_copy.png)

### transpose_input:
![transpose_input](./pngs/output_transpose_input.png)

### vertical_transpose_input:
![vertical_transpose_input](./pngs/output_vertical_transpose_input.png)

### channel_shift_left:
![channel_shift_left](./pngs/output_channel_shift_left.png)

### channel_shift_right:
![channel_shift_right](./pngs/output_channel_shift_right.png)

### half_life_right:
![half_life_right](./pngs/output_half_life_right.png)

### prism_burst:
![prism_burst](./pngs/output_prism_burst.png)

### noise:
![noise](./pngs/output_noise.png)

### compression_ghost:
![compression_ghost](./pngs/output_compression_ghost.png)

### all:
![all](./pngs/output_all.png)

