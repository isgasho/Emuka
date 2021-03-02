use std::convert::TryInto;

use uuid::Uuid;
use warp::Filter;

use crate::{emulators::EmulatorJoypadInput, game::{GameFromFile, SaveFile}};

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
    pub screen: Option<String>
}

impl From<Option<Vec<u8>>> for ScreenDataApi {
    fn from(data: Option<Vec<u8>>) -> Self {
        Self {
            screen: data.map(
                |bytes| base64::encode(bytes)
            )
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

