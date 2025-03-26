use sha1::{Digest, Sha1};
use std::{
    error::Error,
    fs::File,
    io::{Cursor, Read, Seek},
    os::unix::fs::MetadataExt,
};

const DEFAULT_VERSION: u32 = 2;

#[derive(Debug)]
pub struct Index {
    version: u32,
    pub entries: Vec<Entry>,
}

impl Index {
    pub fn empty() -> Self {
        Index {
            version: DEFAULT_VERSION,
            entries: Vec::new(),
        }
    }

    pub fn parse(buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let mut cursor = Cursor::new(buf);
        // Skip signature (DIRC)
        cursor.set_position(4);

        let mut version = [0u8; 4];
        cursor.read_exact(&mut version)?;
        let version = u32::from_be_bytes(version);

        let mut entries_len = [0u8; 4];
        cursor.read_exact(&mut entries_len)?;
        let entries_len = u32::from_be_bytes(entries_len) as usize;

        let mut entries = Vec::with_capacity(entries_len);
        for _ in 0..entries_len {
            let entry = Entry::parse(&mut cursor)?;
            entries.push(entry);
        }

        Ok(Index { version, entries })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut contents: Vec<u8> = Vec::new();

        // Header
        contents.extend("DIRC".as_bytes());
        contents.extend(self.version.to_be_bytes());
        let entries_len: u32 = self.entries.len().try_into()?;
        contents.extend(entries_len.to_be_bytes());

        // Index entries
        for entry in &self.entries {
            entry.serialize(&mut contents);
        }

        let checksum = Sha1::digest(&contents);
        contents.extend(checksum);

        Ok(contents)
    }
}

#[derive(PartialEq, Debug)]
pub struct Entry {
    ctime: i64,
    mtime: i64,
    dev: u64,
    ino: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    size: u64,
    object_id: [u8; 20],
    pub filename: String,
}

impl Entry {
    pub fn from(filename: impl AsRef<str>, file: &mut File) -> Result<Self, Box<dyn Error>> {
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let object_id: [u8; 20] = Sha1::digest(content).try_into()?;

        let metadata = file.metadata()?;
        Ok(Entry {
            ctime: metadata.ctime(),
            mtime: metadata.mtime(),
            dev: metadata.dev(),
            ino: metadata.ino(),
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            size: metadata.size(),
            object_id,
            filename: filename.as_ref().to_string(),
        })
    }

    pub fn parse(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let mut ctime = [0u8; 8];
        cursor.read_exact(&mut ctime)?;
        let ctime = i64::from_be_bytes(ctime);

        let mut mtime = [0u8; 8];
        cursor.read_exact(&mut mtime)?;
        let mtime = i64::from_be_bytes(mtime);

        let mut dev = [0u8; 8];
        cursor.read_exact(&mut dev)?;
        let dev = u64::from_be_bytes(dev);

        let mut ino = [0u8; 8];
        cursor.read_exact(&mut ino)?;
        let ino = u64::from_be_bytes(ino);

        let mut mode = [0u8; 4];
        cursor.read_exact(&mut mode)?;
        let mode = u32::from_be_bytes(mode);

        let mut uid = [0u8; 4];
        cursor.read_exact(&mut uid)?;
        let uid = u32::from_be_bytes(uid);

        let mut gid = [0u8; 4];
        cursor.read_exact(&mut gid)?;
        let gid = u32::from_be_bytes(gid);

        let mut size = [0u8; 8];
        cursor.read_exact(&mut size)?;
        let size = u64::from_be_bytes(size);

        let mut object_id = [0u8; 20];
        cursor.read_exact(&mut object_id)?;

        let mut flag = [0u8; 2];
        cursor.read_exact(&mut flag)?;
        let flag = u16::from_be_bytes(flag);
        let filename_len = usize::try_from(flag & 0xFFF)?;

        let mut filename = vec![0; filename_len];
        cursor.read_exact(&mut filename)?;
        let filename = String::from_utf8(filename)?;

        // Skip null byte and padding
        let padding_len = (8 - (cursor.position() % 8)) % 8;
        cursor.seek_relative(1 + padding_len as i64)?;

        Ok(Entry {
            ctime,
            mtime,
            dev,
            ino,
            mode,
            uid,
            gid,
            size,
            object_id,
            filename,
        })
    }

    pub fn serialize(&self, buf: &mut Vec<u8>) {
        buf.extend(self.ctime.to_be_bytes());
        buf.extend(self.mtime.to_be_bytes());
        buf.extend(self.dev.to_be_bytes());
        buf.extend(self.ino.to_be_bytes());
        buf.extend(self.mode.to_be_bytes());
        buf.extend(self.uid.to_be_bytes());
        buf.extend(self.gid.to_be_bytes());
        buf.extend(self.size.to_be_bytes());
        buf.extend(self.object_id);

        let mut flag: u16 = 0;
        if self.filename.len() < 0xFFF {
            flag |= self.filename.len() as u16;
        } else {
            flag |= 0xFFF;
        }
        buf.extend(flag.to_be_bytes());

        buf.extend(self.filename.as_bytes());
        buf.push(0);
        let padding_len = (8 - (buf.len() % 8)) % 8;
        let padding = vec![0; padding_len];
        buf.extend(padding);
    }
}
