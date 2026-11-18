//! Read only the PE headers and import names. The inspection never loads the target.
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

const AMD64: u16 = 0x8664;

#[derive(Debug)]
pub struct Image {
    pub machine: u16,
    pub imports: Vec<String>,
    pub delayed: Vec<String>,
}

struct Section {
    address: u32,
    size: u32,
    file: u32,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn read_at<const N: usize>(reader: &mut (impl Read + Seek), offset: u64) -> io::Result<[u8; N]> {
    let mut data = [0; N];
    reader.seek(SeekFrom::Start(offset))?;
    reader.read_exact(&mut data)?;
    Ok(data)
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn resolve(sections: &[Section], address: u32, length: u32) -> io::Result<(u64, u32)> {
    for section in sections {
        if let Some(relative) = address.checked_sub(section.address)
            && relative < section.size
            && length <= section.size - relative
        {
            return Ok((
                u64::from(section.file) + u64::from(relative),
                section.size - relative,
            ));
        }
    }
    Err(invalid("endereço da tabela de importação fora do PE"))
}

fn imports<const N: usize>(
    reader: &mut (impl Read + Seek),
    sections: &[Section],
    rva: u32,
    size: u32,
    name_offset: usize,
) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    if rva == 0 {
        return Ok(names);
    }
    let (offset, available) = resolve(sections, rva, N as u32)?;
    for index in 0..(size.min(available).min(4096 * N as u32) / N as u32) {
        let entry = read_at::<N>(reader, offset + u64::from(index) * N as u64)?;
        if entry.iter().all(|byte| *byte == 0) {
            names.sort_unstable();
            names.dedup();
            return Ok(names);
        }
        if N == 32 && u32_at(&entry, 0) & 1 == 0 {
            return Err(invalid(
                "tabela de importação tardia sem endereços relativos",
            ));
        }
        let (start, remaining) = resolve(sections, u32_at(&entry, name_offset), 1)?;
        reader.seek(SeekFrom::Start(start))?;
        let mut raw = [0; 256];
        let limit = remaining.min(raw.len() as u32) as usize;
        reader.read_exact(&mut raw[..limit])?;
        let end = raw[..limit]
            .iter()
            .position(|&byte| byte == 0)
            .ok_or_else(|| invalid("nome de DLL incompleto"))?;
        let name = &raw[..end];
        if !name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(byte))
        {
            return Err(invalid("nome de DLL inválido"));
        }
        let name = String::from_utf8_lossy(name).to_ascii_lowercase();
        if !name.ends_with(".dll") {
            return Err(invalid("importação sem extensão DLL"));
        }
        names.push(name);
    }
    Err(invalid("tabela de importação sem término"))
}

pub fn inspect(path: &Path) -> io::Result<Image> {
    decode(&mut File::open(path)?)
}

fn decode(reader: &mut (impl Read + Seek)) -> io::Result<Image> {
    let dos = read_at::<64>(reader, 0)?;
    if &dos[..2] != b"MZ" {
        return Err(invalid("o arquivo não é um executável Windows"));
    }
    let header = u64::from(u32_at(&dos, 0x3c));
    let nt = read_at::<24>(reader, header)?;
    if &nt[..4] != b"PE\0\0" {
        return Err(invalid("cabeçalho PE inválido"));
    }
    let machine = u16_at(&nt, 4);
    let count = u16_at(&nt, 6);
    let optional_size = u64::from(u16_at(&nt, 20));
    if machine != AMD64 || count == 0 || count > 96 || optional_size < 112 {
        return Err(invalid(
            "este pacote exige um executável Windows x64 válido",
        ));
    }
    let optional_header = header + 24;
    let head = read_at::<112>(reader, optional_header)?;
    if u16_at(&head, 0) != 0x20b {
        return Err(invalid("o executável não contém um cabeçalho PE32+"));
    }
    let directories = u64::from(u32_at(&head, 108));
    let mut sections = Vec::with_capacity(count as usize);
    for index in 0..count {
        let bytes = read_at::<40>(
            reader,
            optional_header + optional_size + u64::from(index) * 40,
        )?;
        sections.push(Section {
            address: u32_at(&bytes, 12),
            size: u32_at(&bytes, 16),
            file: u32_at(&bytes, 20),
        });
    }
    let mut directory = |index: u64| -> io::Result<(u32, u32)> {
        if index >= directories {
            return Ok((0, 0));
        }
        if 112 + (index + 1) * 8 > optional_size {
            return Err(invalid("diretório PE truncado"));
        }
        let data = read_at::<8>(reader, optional_header + 112 + index * 8)?;
        Ok((u32_at(&data, 0), u32_at(&data, 4)))
    };
    let normal = directory(1)?;
    let delay = directory(13)?;
    Ok(Image {
        machine,
        imports: imports::<20>(reader, &sections, normal.0, normal.1, 12)?,
        delayed: imports::<32>(reader, &sections, delay.0, delay.1, 4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_regular_import_from_a_minimal_pe() {
        let mut data = [0u8; 1024];
        data[..2].copy_from_slice(b"MZ");
        data[0x3c] = 0x80;
        data[0x80..0x84].copy_from_slice(b"PE\0\0");
        data[0x84..0x86].copy_from_slice(&AMD64.to_le_bytes());
        data[0x86] = 1;
        data[0x94] = 240;
        data[0x98..0x9a].copy_from_slice(&0x20bu16.to_le_bytes());
        data[0x104] = 16;
        data[0x110..0x114].copy_from_slice(&0x1000u32.to_le_bytes());
        data[0x114] = 40;
        data[0x194..0x198].copy_from_slice(&0x1000u32.to_le_bytes());
        data[0x198..0x19c].copy_from_slice(&512u32.to_le_bytes());
        data[0x19c..0x1a0].copy_from_slice(&512u32.to_le_bytes());
        data[524..528].copy_from_slice(&0x1040u32.to_le_bytes());
        data[576..592].copy_from_slice(b"VCRUNTIME140.dll");
        let result = decode(&mut Cursor::new(data)).unwrap();
        assert_eq!(result.imports, ["vcruntime140.dll"]);
        assert!(result.delayed.is_empty());
        data[524..528].copy_from_slice(&0xffff_ffffu32.to_le_bytes());
        assert!(decode(&mut Cursor::new(data)).is_err());
    }

    #[test]
    fn malformed_and_non_x64_images_are_rejected() {
        assert!(decode(&mut Cursor::new(b"not an executable")).is_err());
        let mut malformed = [0; 128];
        malformed[..2].copy_from_slice(b"MZ");
        malformed[0x3c] = 64;
        malformed[64..68].copy_from_slice(b"PE\0\0");
        assert!(decode(&mut Cursor::new(malformed)).is_err());
    }
}
