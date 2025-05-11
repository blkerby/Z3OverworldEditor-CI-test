use anyhow::{bail, ensure, Context, Result};
use iced::{
    widget::{button, column, container, horizontal_space, row, text},
    Element,
};
use log::info;
use std::path::Path;

use crate::{
    message::Message,
    persist::save_project,
    state::{ColorRGB, ColorValue, EditorState, Palette, PaletteId, Tile},
    view::modal_background_style,
};

type PcAddr = usize;

pub fn snes2pc(addr: usize) -> usize {
    addr >> 1 & 0x3F8000 | addr & 0x7FFF
}

#[derive(Clone)]
pub struct Rom {
    pub data: Vec<u8>,
}

impl Rom {
    pub fn new(data: Vec<u8>) -> Self {
        Rom { data }
    }

    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0xFF);
    }

    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)
            .with_context(|| format!("Unable to load ROM at path {}", path.display()))?;
        Ok(Rom::new(data))
    }

    pub fn read_u8(&self, addr: usize) -> Result<isize> {
        ensure!(addr + 1 <= self.data.len(), "read_u8 address out of bounds");
        Ok(self.data[addr] as isize)
    }

    pub fn read_u16(&self, addr: usize) -> Result<isize> {
        ensure!(
            addr + 2 <= self.data.len(),
            "read_u16 address out of bounds"
        );
        let b0 = self.data[addr] as isize;
        let b1 = self.data[addr + 1] as isize;
        Ok(b0 | b1 << 8)
    }

    pub fn read_u24(&self, addr: usize) -> Result<isize> {
        ensure!(
            addr + 3 <= self.data.len(),
            "read_u24 address out of bounds"
        );
        let b0 = self.data[addr] as isize;
        let b1 = self.data[addr + 1] as isize;
        let b2 = self.data[addr + 2] as isize;
        Ok(b0 | b1 << 8 | b2 << 16)
    }

    pub fn read_n(&self, addr: usize, n: usize) -> Result<&[u8]> {
        ensure!(addr + n <= self.data.len(), "read_n address out of bounds");
        Ok(&self.data[addr..(addr + n)])
    }

    // pub fn save(&self, path: &Path) -> Result<()> {
    //     std::fs::write(path, &self.data)
    //         .with_context(|| format!("Unable to save ROM at path {}", path.display()))?;
    //     Ok(())
    // }

    // pub fn write_u8(&mut self, addr: usize, x: isize) -> Result<()> {
    //     ensure!(
    //         addr + 1 <= self.data.len(),
    //         "write_u8 address out of bounds"
    //     );
    //     ensure!(x >= 0 && x <= 0xFF, "write_u8 data does not fit");
    //     self.data[addr] = x as u8;
    //     Ok(())
    // }

    // pub fn write_u16(&mut self, addr: usize, x: isize) -> Result<()> {
    //     ensure!(
    //         addr + 2 <= self.data.len(),
    //         "write_u16 address out of bounds"
    //     );
    //     ensure!(x >= 0 && x <= 0xFFFF, "write_u16 data does not fit");
    //     self.write_u8(addr, x & 0xFF)?;
    //     self.write_u8(addr + 1, x >> 8)?;
    //     Ok(())
    // }

    // pub fn write_u24(&mut self, addr: usize, x: isize) -> Result<()> {
    //     ensure!(
    //         addr + 3 <= self.data.len(),
    //         "write_u24 address out of bounds"
    //     );
    //     ensure!(x >= 0 && x <= 0xFFFFFF, "write_u24 data does not fit");
    //     self.write_u8(addr, x & 0xFF)?;
    //     self.write_u8(addr + 1, (x >> 8) & 0xFF)?;
    //     self.write_u8(addr + 2, x >> 16)?;
    //     Ok(())
    // }

    // pub fn write_n(&mut self, addr: usize, x: &[u8]) -> Result<()> {
    //     ensure!(
    //         addr + x.len() <= self.data.len(),
    //         "write_n address out of bounds"
    //     );
    //     for i in 0..x.len() {
    //         self.write_u8(addr + i, x[i] as isize)?;
    //     }
    //     Ok(())
    // }
}

pub fn import_rom_view(state: &EditorState) -> Element<Message> {
    container(
        column![
            text("Import project from ROM?"),
            text("This may replace existing project data, including palettes, tilesets, and screens."),
            row![
                button(text("Import from ROM"))
                .style(button::danger)
                .on_press(Message::ImportROM),
                horizontal_space(),
                button(text("Cancel"))
                .style(button::secondary)
                .on_press(Message::CloseDialogue),
            ]
        ]
        .spacing(15),
    )
    .width(450)
    .padding(25)
    .style(modal_background_style)
    .into()
}

fn import_palette(
    state: &mut EditorState,
    rom: &Rom,
    addr: PcAddr,
    size: usize,
    name: &str,
    id: PaletteId,
) -> Result<()> {
    let mut colors = [(0, 0, 0); 16];
    for i in 0..size {
        let c = rom.read_u16(addr + i * 2)?;
        let r = c & 31;
        let g = (c >> 5) & 31;
        let b = (c >> 10) & 31;
        colors[i + 1] = (r as ColorValue, g as ColorValue, b as ColorValue);
    }
    state.palettes.push(Palette {
        modified: true,
        name: name.to_string(),
        id,
        colors,
        tiles: vec![[[0; 8]; 8]; 16],
    });
    Ok(())
}

fn import_all_palettes(state: &mut EditorState, rom: &Rom, mut id: PaletteId) -> Result<()> {
    let palette_groups = [
        ("Main", 0xDE6C8, 6, 5),
        ("Aux", 0xDE86C, 20, 3),
        ("Animated", 0xDE604, 14, 1),
    ];

    for (group_name, base_addr, cnt_pal, cnt_rows) in palette_groups {
        for i in 0..cnt_pal {
            for j in 0..cnt_rows {
                let name = format!("{} {:x}-{}", group_name, i, j);
                let addr = base_addr + ((i * cnt_rows + j) * 7) * 2;
                let size = 7;
                import_palette(state, rom, addr, size, &name, id)?;
                id += 1;
            }
        }
    }
    Ok(())
}

fn decompress(rom: &Rom, mut addr: PcAddr) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let byte = rom.read_u8(addr)?;
        addr += 1;
        if byte == 0xFF {
            return Ok(out);
        }
        let mut block_type = byte >> 5;
        let size: usize;
        if block_type != 7 {
            size = ((byte & 0x1F) + 1) as usize;
        } else {
            size = (((byte & 3) << 8 | rom.read_u8(addr)?) + 1) as usize;
            addr += 1;
            block_type = (byte >> 2) & 7;
        }

        match block_type {
            0 => {
                // Raw block
                out.extend(rom.read_n(addr, size)?);
                addr += size;
            }
            1 => {
                // Byte-level RLE block
                let value = rom.read_u8(addr)? as u8;
                addr += 1;
                out.extend(&vec![value; size]);
            }
            2 => {
                // Word-level RLE block
                let b0 = rom.read_u8(addr)? as u8;
                let b1 = rom.read_u8(addr + 1)? as u8;
                addr += 2;
                out.extend(&[b0, b1].repeat(size >> 1));
                if size & 1 == 1 {
                    out.push(b0);
                }
            }
            3 => {
                // Incrementing sequence
                let mut b = rom.read_u8(addr)? as u8;
                addr += 1;
                for _ in 0..size {
                    out.push(b);
                    b = b.saturating_add(1);
                }
            }
            4 => {
                // Copy earlier output, with absolute offset:
                let offset = rom.read_u16(addr)? as usize;
                assert!(offset < out.len());
                addr += 2;
                for i in offset..(offset + size) {
                    out.push(out[i]);
                }
            }
            _ => {
                bail!("Unexpected/impossible block type: {block_type}");
            }
        }
    }

    Ok(vec![])
}

fn load_graphics(state: &mut EditorState, rom: &Rom) -> Result<()> {
    let gfx_low = snes2pc(rom.read_u16(0x67DA)? as usize);
    let gfx_high = snes2pc(rom.read_u16(0x67D5)? as usize);
    let gfx_bank = snes2pc(rom.read_u16(0x67D0)? as usize);

    let mut tiles: Vec<Tile> = vec![];
    for i in 0..113 {
        let addr_low = rom.read_u8(gfx_low + i)? as usize;
        let addr_high = rom.read_u8(gfx_high + i)? as usize;
        let addr_bank = rom.read_u8(gfx_bank + i)? as usize;
        let addr = snes2pc(addr_low | (addr_high << 8) | (addr_bank << 16));
        let data = decompress(rom, addr)?;
        if data.len() != 0x600 {
            bail!("Unexpected graphics sheet length: {}", data.len());
        }

        for j in 0..64 {
            let mut tile: Tile = [[0; 8]; 8];
            for y in 0..8 {
                for x in 0..8 {
                    let c0 = (data[j * 24 + y * 2] >> (7 - x)) & 1;
                    let c1 = (data[j * 24 + y * 2 + 1] >> (7 - x)) & 1;
                    let c2 = (data[j * 24 + y + 16] >> (7 - x)) & 1;
                    let c = c0 | (c1 << 1) | (c2 << 2);
                    tile[y][x] = c;
                }
            }
            tiles.push(tile);
        }
    }
    state.palettes[0].tiles = tiles;
    state.palettes[0].modified = true;
    Ok(())
}

pub fn import_from_rom(state: &mut EditorState, path: &Path) -> Result<()> {
    let rom_bytes = std::fs::read(path)?;
    let rom = Rom::new(rom_bytes);
    let starting_palette_id = 100;
    info!("Importing from ROM at {}", path.display());
    // import_all_palettes(state, &rom, starting_palette_id)?;
    load_graphics(state, &rom)?;
    save_project(state)?;
    Ok(())
}
