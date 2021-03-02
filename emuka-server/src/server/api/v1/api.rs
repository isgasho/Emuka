use std::convert::TryInto;

use uuid::Uuid;
use warp::Filter;

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
    pub screen: Option<String>,
    pub width: u32,
    pub height: u32
}

impl From<Option<ScreenData>> for ScreenDataApi {
    fn from(data: Option<ScreenData>) -> Self {
        match data {
            Some(screen_data) => Self {
                width: screen_data.width,
                height: screen_data.height,
                screen: Some(base64::encode(screen_data.data))
            },
            None => Self {
                screen: None,
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
    pub data: Option<String>
}

