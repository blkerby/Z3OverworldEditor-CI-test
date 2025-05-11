use anyhow::{bail, ensure, Context, Result};
use iced::{
    widget::{button, column, container, horizontal_space, row, text},
    Element,
};
use log::info;
use std::{
    fmt::Display,
    ops::{Add, AddAssign},
    path::Path,
};

use crate::{
    message::Message,
    persist::save_project,
    state::{ColorRGB, ColorValue, EditorState, Palette, PaletteId, Tile},
    update::update_palette_order,
    view::modal_background_style,
};

// From past experience, it's a very common mistake to mix up SNES addresses
// with "PC" addresses (byte index into the ROM file). So we use type-safe wrappers
// to make these harder to mess up:
#[derive(Copy, Clone)]
struct PcAddr(u32);

#[derive(Copy, Clone)]
struct SnesAddr(u32);

macro_rules! impl_add {
    ($target_type:ident, $other_type:ident) => {
        impl Add<$other_type> for $target_type {
            type Output = $target_type;

            fn add(self, other: $other_type) -> Self {
                $target_type((self.0 as $other_type + other) as u32)
            }
        }
    };
}

impl_add!(PcAddr, i32);
impl_add!(PcAddr, usize);
impl_add!(SnesAddr, i32);
impl_add!(SnesAddr, usize);

macro_rules! impl_add_assign {
    ($target_type:ident, $other_type:ident) => {
        impl AddAssign<$other_type> for $target_type {
            fn add_assign(&mut self, other: $other_type) {
                self.0 = (self.0 as $other_type + other) as u32;
            }
        }
    };
}

impl_add_assign!(PcAddr, i32);
impl_add_assign!(PcAddr, usize);
impl_add_assign!(SnesAddr, i32);
impl_add_assign!(SnesAddr, usize);

impl Display for PcAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0)?;
        Ok(())
    }
}

impl Display for SnesAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:X}", self.0)?;
        Ok(())
    }
}

impl SnesAddr {
    fn from_bytes(bank: u8, high: u8, low: u8) -> Self {
        Self((bank as u32) << 16 | (high as u32) << 8 | low as u32)
    }

    fn from_bank_offset(bank: u8, offset: u16) -> Self {
        Self((bank as u32) << 16 | offset as u32)
    }
}

impl From<SnesAddr> for PcAddr {
    fn from(addr: SnesAddr) -> Self {
        // It would be a mistake if we ever reference the lower half of a bank
        // (which would often map to RAM, registers, etc. rather than to ROM).
        // Likewise we don't reference the upper/mirrored banks:
        assert!(addr.0 & 0x808000 == 0x8000);

        PcAddr(addr.0 >> 1 & 0x3F8000 | addr.0 & 0x7FFF)
    }
}

// Addresses of where certain data is located in the ROM. Some of these vary
// between JP and US versions.
struct Constants {
    main_palettes_addr: SnesAddr,
    aux_palettes_addr: SnesAddr,
    animated_palettes_addr: SnesAddr,
    gfx_bank_addr: SnesAddr,
    gfx_high_addr: SnesAddr,
    gfx_low_addr: SnesAddr,
    tiles_16x16_addr: SnesAddr,
}

impl Constants {
    fn jp() -> Self {
        Constants {
            main_palettes_addr: SnesAddr(0x1BE6C8),
            aux_palettes_addr: SnesAddr(0x1BE86C),
            animated_palettes_addr: SnesAddr(0x1BE604),
            gfx_bank_addr: SnesAddr(0x00E7D0),
            gfx_high_addr: SnesAddr(0x00E7D5),
            gfx_low_addr: SnesAddr(0x00E7DA),
            tiles_16x16_addr: SnesAddr(0x0F8000),
        }
    }

    fn us() -> Self {
        Constants {
            main_palettes_addr: SnesAddr(0x1BE6C8),
            aux_palettes_addr: SnesAddr(0x1BE86C),
            animated_palettes_addr: SnesAddr(0x1BE604),
            gfx_bank_addr: SnesAddr(0x00E790),
            gfx_high_addr: SnesAddr(0x00E795),
            gfx_low_addr: SnesAddr(0x00E79A),
            tiles_16x16_addr: SnesAddr(0x0F8000),
        }
    }

    fn auto(rom: &Rom) -> Result<Self> {
        if rom.read_u16(SnesAddr(0x00E7D2).into())? == 0xCA85 {
            info!("JP ROM format detected.");
            Ok(Constants::jp())
        } else if rom.read_u16(SnesAddr(0x00E792).into())? == 0xCA85 {
            info!("US ROM format detected.");
            Ok(Constants::us())
        } else {
            bail!("Unknown ROM format.");
        }
    }
}

#[derive(Clone)]
struct Rom {
    pub data: Vec<u8>,
}

// struct Tile16 {
//     top_left:
// }

pub struct Importer<'a> {
    state: &'a mut EditorState,
    constants: Constants,
    rom: Rom,
    // tiles16: Vec<Tile16>,
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

    pub fn read_u8(&self, addr: PcAddr) -> Result<u8> {
        ensure!(
            (addr.0 as usize) < self.data.len(),
            "read_u8 address out of bounds"
        );
        Ok(self.data[addr.0 as usize] as u8)
    }

    pub fn read_u16(&self, addr: PcAddr) -> Result<u16> {
        ensure!(
            addr.0 as usize + 1 < self.data.len(),
            "read_u16 address out of bounds"
        );
        let b0 = self.data[addr.0 as usize] as u16;
        let b1 = self.data[addr.0 as usize + 1] as u16;
        Ok(b0 | b1 << 8)
    }

    pub fn read_n(&self, addr: PcAddr, n: usize) -> Result<&[u8]> {
        ensure!(
            addr.0 as usize + n <= self.data.len(),
            "read_n address out of bounds"
        );
        Ok(&self.data[addr.0 as usize..(addr.0 as usize + n)])
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

impl<'a> Importer<'a> {
    pub fn import(state: &'a mut EditorState, path: &Path) -> Result<()> {
        info!("Importing from ROM at {}", path.display());
        let mut importer = Self::new(state, path)?;
        importer.import_all()?;
        Ok(())
    }

    fn new(state: &'a mut EditorState, path: &Path) -> Result<Self> {
        let rom_bytes = std::fs::read(path)?;
        let rom = Rom::new(rom_bytes);
        Ok(Self {
            state,
            constants: Constants::auto(&rom)?,
            rom,
        })
    }

    fn import_all(&mut self) -> Result<()> {
        let starting_palette_id = 100;
        self.import_all_palettes(starting_palette_id)?;
        self.load_graphics()?;
        save_project(self.state)?;
        Ok(())
    }

    fn import_palette(
        &mut self,
        addr: PcAddr,
        size: usize,
        name: &str,
        id: PaletteId,
    ) -> Result<()> {
        let mut colors = [(0, 0, 0); 16];
        for i in 0..size {
            let c = self.rom.read_u16(addr + i * 2)?;
            let r = c & 31;
            let g = (c >> 5) & 31;
            let b = (c >> 10) & 31;
            colors[i + 1] = (r as ColorValue, g as ColorValue, b as ColorValue);
        }
        self.state.palettes.push(Palette {
            modified: true,
            name: name.to_string(),
            id,
            colors,
            tiles: vec![[[0; 8]; 8]; 16],
        });
        Ok(())
    }

    fn import_all_palettes(&mut self, mut id: PaletteId) -> Result<()> {
        let palette_groups = [
            ("Main", self.constants.main_palettes_addr, 6, 5),
            ("Aux", self.constants.aux_palettes_addr, 20, 3),
            ("Animated", self.constants.animated_palettes_addr, 14, 1),
        ];

        self.state.palettes.clear();
        for (group_name, base_addr, cnt_pal, cnt_rows) in palette_groups {
            let base_addr: PcAddr = base_addr.into();
            for i in 0..cnt_pal {
                for j in 0..cnt_rows {
                    let name = format!("{} {:x}-{}", group_name, i, j);
                    let addr = base_addr + ((i * cnt_rows + j) * 7) * 2;
                    let size = 7;
                    self.import_palette(addr, size, &name, id)?;
                    id += 1;
                }
            }
        }
        update_palette_order(self.state);
        Ok(())
    }

    fn load_graphics(&mut self) -> Result<()> {
        let rom = &self.rom;
        let gfx_bank = rom.read_u16(self.constants.gfx_bank_addr.into())?;
        let gfx_high = rom.read_u16(self.constants.gfx_high_addr.into())?;
        let gfx_low = rom.read_u16(self.constants.gfx_low_addr.into())?;

        let mut tiles: Vec<Tile> = vec![];
        for i in 0..113 {
            let bank = rom.read_u8(SnesAddr::from_bank_offset(0x00, gfx_bank + i).into())?;
            let high = rom.read_u8(SnesAddr::from_bank_offset(0x00, gfx_high + i).into())?;
            let low = rom.read_u8(SnesAddr::from_bank_offset(0x00, gfx_low + i).into())?;
            let addr = SnesAddr::from_bytes(bank, high, low);
            let data = decompress(rom, addr.into())?;
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
        self.state.palettes[0].tiles = tiles;
        self.state.palettes[0].modified = true;
        Ok(())
    }

    fn load_16x16_tiles(&mut self) -> Result<()> {
        Ok(())
    }
}

fn decompress(rom: &Rom, mut addr: PcAddr) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let byte = rom.read_u8(addr)? as isize;
        addr += 1;
        if byte == 0xFF {
            return Ok(out);
        }
        let mut block_type = byte >> 5;
        let size: usize;
        if block_type != 7 {
            size = ((byte & 0x1F) + 1) as usize;
        } else {
            size = (((byte & 3) << 8 | rom.read_u8(addr)? as isize) + 1) as usize;
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
                let value = rom.read_u8(addr)?;
                addr += 1;
                out.extend(&vec![value; size]);
            }
            2 => {
                // Word-level RLE block
                let b0 = rom.read_u8(addr)?;
                let b1 = rom.read_u8(addr + 1)?;
                addr += 2;
                out.extend(&[b0, b1].repeat(size >> 1));
                if size & 1 == 1 {
                    out.push(b0);
                }
            }
            3 => {
                // Incrementing sequence
                let mut b = rom.read_u8(addr)?;
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
}
