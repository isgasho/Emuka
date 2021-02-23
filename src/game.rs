use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom, Write}, path::PathBuf};
use std::path::Path;
use eyre::Result;
use core::fmt::Debug;

pub trait Game: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn data(&self) -> &[u8];
    fn path(&self) -> Option<Box<Path>>;
}

#[derive(Debug)]
pub struct GameFromFile  {
    name: String,
    data: Vec<u8>,
    path: PathBuf
}

impl GameFromFile {
    pub fn new <S: AsRef<str>, P: AsRef<Path>> (name: S, path: P) -> Result<Self> {
        let file = File::open(&path)?;
        let mut buffer = BufReader::new(file);
        let mut data = Vec::<u8>::new();
        buffer.read_to_end(&mut data)?;

        Ok(Self {
            name: name.as_ref().to_string(),
            data,
            path: path.as_ref().to_owned()
        })
    }
}

impl Game for GameFromFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn path(&self) -> Option<Box<Path>> {
        Some(self.path.clone().into_boxed_path())
    }
}


#[derive(Debug)]
pub struct GameFromData {
    name: String,
    data: Vec<u8>
}

impl GameFromData {
    pub fn new <S: AsRef<str>, D: AsRef<[u8]>> (name: S, data: D) -> Self {
        Self {
            name: name.as_ref().to_string(),
            data: data.as_ref().to_owned()
        }
    }    
}

impl Game for GameFromData {
    fn name(&self) -> &str {
        &self.name
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn path(&self) -> Option<Box<Path>> {
        None
    }
}


pub trait Save: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn data(&self) -> &[u8];
    fn path(&self) -> Option<Box<Path>>;
    
    fn can_write(&self) -> bool {
        false
    }

    fn write(&mut self, data: &[u8]) -> Result<()>;
}

#[derive(Debug)]
pub struct SaveFile {
    name: String,
    data: Vec<u8>,
    path: Option<PathBuf>,
    file: File,
    writable: bool
}


impl SaveFile {
    pub fn new <S: AsRef<str>, P: AsRef<Path>> (name: S, path: P) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut buffer = BufReader::new(file.try_clone()?);
        let mut data = Vec::<u8>::new();
        buffer.read_to_end(&mut data)?;
        file.seek(SeekFrom::Start(0))?;
        let writable = !file.metadata()?.permissions().readonly();

        Ok(Self {
            name: name.as_ref().to_string(),
            data,
            path: Some(path.as_ref().to_owned()),
            file,
            writable
        })
    }
}


impl Save for SaveFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn path(&self) -> Option<Box<Path>> {
        self.path.clone().and_then(|value| Some(value.into_boxed_path()))
    }

    fn can_write(&self) -> bool {
        self.writable        
    }

    fn write (&mut self, data: &[u8]) -> Result<()> {
        self.file.set_len(0)?;
        self.file.write_all(data.as_ref())?;
        Ok(())
    }
}

