use std::convert::TryInto;

use uuid::Uuid;
use warp::Filter;
use avro_rs::{Schema};
use lazy_static::lazy_static;

use crate::{emulators::{EmulatorJoypadInput, ScreenData}, game::{GameFromFile, SaveFile}};



#[derive(Debug, Deserialize, Clone)]
pub struct GameFromFileApi {
    pub path: String
}


impl TryInto<GameFromFile> for GameFromFileApi {
    type Error = eyre::Report;

    fn try_into(self) -> Result<GameFromFile, Self::Error> {
        Ok(GameFromFile::new(&self.path, &self.path)?)
    }
}

pub fn post_json <T: Send + Sync + serde::de::DeserializeOwned> () -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}


#[derive(Debug, Deserialize, Clone)]
pub struct SaveFromFileApi {
    pub path: String
}


impl TryInto<SaveFile> for SaveFromFileApi {
    type Error = eyre::Report;

    fn try_into(self) -> Result<SaveFile, Self::Error> {
        Ok(SaveFile::new(&self.path, &self.path)?)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmulatorJoypadInputApi {
    pub input: EmulatorJoypadInput,
    pub pressed: bool
}

impl Into<(EmulatorJoypadInput, bool)> for EmulatorJoypadInputApi {
    fn into(self) -> (EmulatorJoypadInput, bool) {
        (self.input, self.pressed)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ScreenDataApi {
    pub screen: Vec<u8>,
    pub width: u32,
    pub height: u32
}

lazy_static! {
    static ref RAW_SCHEMA_SCREEN_DATA_API: &'static str = r#"
        {
            "type": "record",
            "name": "ScreenData",
            "fields": [
                {"name": "screen", "type": "bytes"},
                {"name": "width", "type": "int", "default": 0},
                {"name": "height", "type": "int", "default": 0}
            ]
        }
    "#;

    pub static ref SCREEN_DATA_API_SCHEMA: Schema = Schema::parse_str(&RAW_SCHEMA_SCREEN_DATA_API).unwrap();
}

impl From<Option<ScreenData>> for ScreenDataApi {
    fn from(data: Option<ScreenData>) -> Self {
        match data {
            Some(screen_data) => Self {
                width: screen_data.width,
                height: screen_data.height,
                screen: screen_data.data
            },
            None => Self {
                screen: Vec::new(),
                width: 0,
                height: 0
            }
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AudioRegisterApi {
    pub id: Uuid
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GetAudioSamplesResponseApi {
    pub data: Vec<u8>
}

lazy_static! {
    static ref RAW_SCHEMA_AUDIO_DATA_API: &'static str = r#"
        {
            "type": "record",
            "name": "AudioData",
            "fields": [
                {"name": "data", "type": "bytes"}
            ]
        }
    "#;

    pub static ref AUDIO_DATA_API_SCHEMA: Schema = Schema::parse_str(&RAW_SCHEMA_AUDIO_DATA_API).unwrap();
}
