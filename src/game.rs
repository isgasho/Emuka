use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom, Write}};
use std::path::Path;
use eyre::Result;

pub trait Game {
    fn name(&self) -> &str;
    fn data(&self) -> &[u8];
}

#[derive(Debug, Clone)]
pub struct GameFromFile  {
    name: String,
    data: Vec<u8>
}

impl GameFromFile {
    pub fn new <S: AsRef<str>, P: AsRef<Path>> (name: S, path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut buffer = BufReader::new(file);
        let mut data = Vec::<u8>::new();
        buffer.read_to_end(&mut data)?;

        Ok(Self {
            name: name.as_ref().to_string(),
            data
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
}



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
}


trait Save {
    fn name(&self) -> &str;
    fn data(&self) -> &[u8];
    
    fn can_write(&self) -> bool {
        false
    }

    fn write <D: AsRef<[u8]>> (&mut self, data: D) -> Result<()>;
}


pub struct SaveFileTrait {
    name: String,
    data: Vec<u8>,
    file: File,
    writable: bool
}





impl SaveFileTrait {
    pub fn new <S: AsRef<str>, P: AsRef<Path>> (name: S, path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = BufReader::new(file.try_clone()?);
        let mut data = Vec::<u8>::new();
        buffer.read_to_end(&mut data)?;
        file.seek(SeekFrom::Start(0))?;
        let writable = !file.metadata()?.permissions().readonly();

        Ok(Self {
            name: name.as_ref().to_string(),
            data,
            file,
            writable
        })
    }
}


impl Save for SaveFileTrait {
    fn name(&self) -> &str {
        &self.name
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn can_write(&self) -> bool {
        self.writable        
    }

    fn write <D: AsRef<[u8]>> (&mut self, data: D) -> Result<()> {
        self.file.set_len(0)?;
        self.file.write_all(data.as_ref())?;
        Ok(())
    }
}

