use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PowerModule {
    pub status: f32,
    pub init: bool,
    pub links: Vec<i32>,
}

impl PowerModule {
    pub fn sanitize(&mut self) {
        if self.status.is_nan() || self.status.is_infinite() {
            self.status = 0.0;
        }
    }

    pub fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write_i16(write, self.links.len() as i16)?;
        for link in &self.links {
            write_i32(write, *link)?;
        }
        write_f32(write, self.status)
    }

    pub fn read<R: Read>(&mut self, read: &mut R) -> io::Result<()> {
        self.links.clear();
        let amount = read_i16(read)?;
        for _ in 0..amount {
            self.links.push(read_i32(read)?);
        }
        self.status = read_f32(read)?;
        self.sanitize();
        Ok(())
    }
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_module_links_and_status_roundtrip_like_java() {
        let module = PowerModule {
            status: 0.75,
            init: true,
            links: vec![10, -20],
        };
        let mut bytes = Vec::new();
        module.write(&mut bytes).unwrap();
        assert_eq!(bytes[0..2], 2i16.to_be_bytes());
        assert_eq!(bytes[2..6], 10i32.to_be_bytes());
        assert_eq!(bytes[6..10], (-20i32).to_be_bytes());

        let mut restored = PowerModule::default();
        restored.read(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.status, 0.75);
        assert_eq!(restored.links, vec![10, -20]);

        let mut bad = Vec::new();
        write_i16(&mut bad, 0).unwrap();
        write_f32(&mut bad, f32::NAN).unwrap();
        restored.read(&mut bad.as_slice()).unwrap();
        assert_eq!(restored.status, 0.0);
    }
}
