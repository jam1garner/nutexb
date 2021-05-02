//use super::enums as gxe;

enum Size {
    Bit4,
    Bit8,
    Bit16,
    Bit32,
}

trait Decode {
    const SIZE: Size;
    fn decode_to_rgba(value: u32) -> u32;
}

fn decode_image_generic<D: Decode>(input: &[u8], width: usize, height: usize) -> Vec<u8> {
    let (block_width, block_height, block_row_stride, block_size) = match D::SIZE {
        Size::Bit4 => (8, 8, 4, 32),
        Size::Bit8 => (8, 4, 8, 32),
        Size::Bit16 => (4, 4, 8, 32),
        Size::Bit32 => (4, 4, 8, 64),
    };
    println!("width: {} / height: {}", width, height);
    let output_row_stride = width * 4;
    let buffer_size = width * height * 4;
    let mut output = Vec::with_capacity(buffer_size);
    let mut block_in = 0;
    let mut block_out = 0;
    output.resize(buffer_size, 0);

    for _block_y in 0..(height / block_height) {
        for _block_x in 0..(width / block_width) {
            let mut px_in = block_in;
            let mut px_out = block_out;
            for _px_y in 0..block_height {
                for px_x in 0..block_width {
                    let value = match D::SIZE {
                        Size::Bit4 => {
                            // x=0 -> high 4 bits, x=1 -> low 4 bits
                            let shift = 4 * ((px_x & 1) ^ 1);
                            ((input[px_in + (px_x / 2)] >> shift) & 0xF) as u32
                        }
                        Size::Bit8 => input[px_in + px_x] as u32,
                        Size::Bit16 => {
                            let a = input[px_in + px_x * 2] as u32;
                            let b = input[px_in + px_x * 2 + 1] as u32;
                            (a << 8) | b
                        }
                        Size::Bit32 => {
                            // ARGB
                            let a = input[px_in + px_x * 2] as u32;
                            let b = input[px_in + px_x * 2 + 1] as u32;
                            let c = input[px_in + px_x * 2 + 32] as u32;
                            let d = input[px_in + px_x * 2 + 33] as u32;
                            (a << 24) | (b << 16) | (c << 8) | d
                        }
                    };
                    let value = D::decode_to_rgba(value);
                    output[px_out..px_out + 4].copy_from_slice(&value.to_be_bytes());
                    px_out += 4;
                }
                px_in += block_row_stride;
                px_out += output_row_stride - (block_width * 4);
            }

            block_in += block_size;
            block_out += 4 * block_width;
        }
        block_out += output_row_stride * (block_height - 1);
    }

    output
}

fn pack_rgba(r: u32, g: u32, b: u32, a: u32) -> u32 {
    (r << 24) | (g << 16) | (b << 8) | a
}

fn extend_3(v: u32) -> u32 {
    (v << 5) | (v << 2) | (v >> 1)
}
fn extend_4(v: u32) -> u32 {
    (v << 4) | v
}
fn extend_5(v: u32) -> u32 {
    (v << 3) | (v >> 2)
}
fn extend_6(v: u32) -> u32 {
    (v << 2) | (v >> 4)
}

struct I4;
impl Decode for I4 {
    const SIZE: Size = Size::Bit4;
    fn decode_to_rgba(value: u32) -> u32 {
        let i = extend_4(value & 0xF);
        pack_rgba(i, i, i, 0xFF)
    }
}

struct I8;
impl Decode for I8 {
    const SIZE: Size = Size::Bit8;
    fn decode_to_rgba(value: u32) -> u32 {
        let i = value & 0xFF;
        pack_rgba(i, i, i, 0xFF)
    }
}

struct IA4;
impl Decode for IA4 {
    const SIZE: Size = Size::Bit8;
    fn decode_to_rgba(value: u32) -> u32 {
        let i = extend_4(value & 0xF);
        let a = extend_4((value >> 4) & 0xF);
        pack_rgba(i, i, i, a)
    }
}

struct IA8;
impl Decode for IA8 {
    const SIZE: Size = Size::Bit16;
    fn decode_to_rgba(value: u32) -> u32 {
        let i = (value >> 8) & 0xFF;
        let a = value & 0xFF;
        pack_rgba(i, i, i, a)
    }
}

struct RGB565;
impl Decode for RGB565 {
    const SIZE: Size = Size::Bit16;
    fn decode_to_rgba(value: u32) -> u32 {
        let r = extend_5((value >> 11) & 0x1F);
        let g = extend_5((value >> 5) & 0x3F);
        let b = extend_5(value & 0x1F);
        pack_rgba(r, g, b, 0xFF)
    }
}

struct RGB5A3;
impl Decode for RGB5A3 {
    const SIZE: Size = Size::Bit16;
    fn decode_to_rgba(value: u32) -> u32 {
        if (value & 0x8000) != 0 {
            let r = extend_5((value >> 10) & 0x1F);
            let g = extend_5((value >> 5) & 0x1F);
            let b = extend_5(value & 0x1F);
            pack_rgba(r, g, b, 0xFF)
        } else {
            let a = extend_3((value >> 12) & 7);
            let r = extend_4((value >> 8) & 0xF);
            let g = extend_4((value >> 4) & 0xF);
            let b = extend_4(value & 0xF);
            pack_rgba(r, g, b, a)
        }
    }
}

struct RGBA8;
impl Decode for RGBA8 {
    const SIZE: Size = Size::Bit32;
    fn decode_to_rgba(value: u32) -> u32 {
        value.rotate_left(8) // ARGB -> RGBA
    }
}


fn avg_1_1(a: u32, b: u32) -> u32 {
    (a + b) / 2
}
fn avg_2_1(a: u32, b: u32) -> u32 {
    (a + a + b) / 3
}

fn calc_cmpr_block(c0: u32, c1: u32) -> [u32; 4] {
    // decode the two colours
    let r0 = extend_5((c0 >> 11) & 0x1F);
    let g0 = extend_6((c0 >> 5) & 0x3F);
    let b0 = extend_5(c0 & 0x1F);
    let rgba0 = pack_rgba(r0, g0, b0, 0xFF);

    let r1 = extend_5((c1 >> 11) & 0x1F);
    let g1 = extend_6((c1 >> 5) & 0x3F);
    let b1 = extend_5(c1 & 0x1F);
    let rgba1 = pack_rgba(r1, g1, b1, 0xFF);

    let (rgba2, rgba3) = if c0 > c1 {
        (pack_rgba(avg_2_1(r0, r1), avg_2_1(g0, g1), avg_2_1(b0, b1), 0xFF),
        pack_rgba(avg_2_1(r1, r0), avg_2_1(g1, g0), avg_2_1(b1, b0), 0xFF))
    } else {
        (pack_rgba(avg_1_1(r0, r1), avg_1_1(g0, g1), avg_1_1(b0, b1), 0xFF),
        pack_rgba(0, 0, 0, 0))
    };

    [rgba0, rgba1, rgba2, rgba3]
}

pub fn decode_image_cmpr(input: &[u8], width: usize, height: usize) -> Vec<u8> {
    let output_row_stride = width * 4;
    let buffer_size = width * height * 4;
    let mut output = Vec::with_capacity(buffer_size);
    let mut in_addr = 0;
    output.resize(buffer_size, 0);

    for outer_y in 0..(height / 8) {
        for outer_x in 0..(width / 8) {
            let outer_pos = 4 * ((width * outer_y * 8) + (outer_x * 8));
            for block_y in 0..2 {
                for block_x in 0..2 {
                    let mut block_pos = outer_pos + 4 * ((width * block_y * 4) + (block_x * 4));

                    // decode block colours
                    let raw0 = ((input[in_addr] as u32) << 8) | (input[in_addr + 1] as u32);
                    let raw1 = ((input[in_addr + 2] as u32) << 8) | (input[in_addr + 3] as u32);
                    let col_array = calc_cmpr_block(raw0, raw1);
                    in_addr += 4;

                    // decode the 4x4 pixels now
                    for px_y in 0..4 {
                        let mut row = input[in_addr + px_y];
                        for _px_x in 0..4 {
                            let idx = row >> 6;
                            let value = col_array[idx as usize];
                            output[block_pos..block_pos + 4].copy_from_slice(&value.to_be_bytes());
                            block_pos += 4;
                            row <<= 2;
                        }
                        block_pos += output_row_stride - (4 * 4);
                    }
                    in_addr += 4;
                }
            }
        }
    }

    output
}

//pub fn get_byte_size(format: gxe::TexFmt, width: usize, height: usize) -> usize {
//    let size = match format {
//        gxe::TexFmt::I4 => I4::SIZE,
//        gxe::TexFmt::I8 => I8::SIZE,
//        gxe::TexFmt::IA4 => IA4::SIZE,
//        gxe::TexFmt::IA8 => IA8::SIZE,
//        gxe::TexFmt::RGB565 => RGB565::SIZE,
//        gxe::TexFmt::RGB5A3 => RGB5A3::SIZE,
//        gxe::TexFmt::RGBA8 => RGBA8::SIZE,
//        gxe::TexFmt::CMPR => Size::Bit4,
//        _ => panic!("bad texture format"),
//    };
//    match size {
//        Size::Bit4 => (width * height) / 2,
//        Size::Bit8 => width * height,
//        Size::Bit16 => width * height * 2,
//        Size::Bit32 => width * height * 4
//    }
//}
//
//pub fn decode_image(format: gxe::TexFmt, input: &[u8], width: usize, height: usize) -> Vec<u8> {
//    match format {
//        gxe::TexFmt::I4 => decode_image_generic::<I4>(input, width, height),
//        gxe::TexFmt::I8 => decode_image_generic::<I8>(input, width, height),
//        gxe::TexFmt::IA4 => decode_image_generic::<IA4>(input, width, height),
//        gxe::TexFmt::IA8 => decode_image_generic::<IA8>(input, width, height),
//        gxe::TexFmt::RGB565 => decode_image_generic::<RGB565>(input, width, height),
//        gxe::TexFmt::RGB5A3 => decode_image_generic::<RGB5A3>(input, width, height),
//        gxe::TexFmt::RGBA8 => decode_image_generic::<RGBA8>(input, width, height),
//        gxe::TexFmt::CMPR => decode_image_cmpr(input, width, height),
//        _ => panic!("bad texture format"),
//    }
//}
