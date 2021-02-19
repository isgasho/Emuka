use std::{convert::TryFrom, ffi::{CStr, CString, c_void}, sync::RwLock};
use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use eyre::*;
use super::bindings;

#[repr(u32)]
#[derive(TryFromPrimitive, Debug)]
pub enum EnvironmentCallbackCmd {
    GetVariable = bindings::RETRO_ENVIRONMENT_GET_VARIABLE
}

#[derive(Debug)]
pub enum EnvironmentCallbackData {
    RetroVariable(RetroVariable)
}

impl EnvironmentCallbackData {
    pub unsafe fn from_cmd_and_void_ptr(cmd: &EnvironmentCallbackCmd, data: *mut c_void) -> Result<Self> {
        use EnvironmentCallbackCmd::*;

        Ok(match cmd {
            GetVariable => Self::RetroVariable(RetroVariable::from_void_ptr(data)?)
        })
    }

    pub unsafe fn repopulate_void_ptr(&self, data: *mut c_void) -> Result<()> {
        Ok(match self {
            EnvironmentCallbackData::RetroVariable(retro_variable) => {
                retro_variable.repopulate_void_ptr(data)?;
            }
        })
    }
}

pub type EnvironmentCallback = fn(cmd: &EnvironmentCallbackCmd, data: &mut EnvironmentCallbackData) -> bool;

lazy_static! {
    static ref ENVIRON_CALLBACK_GLOBAL: RwLock<Option<EnvironmentCallback>> = RwLock::new(None);
}

#[derive(Debug)]
pub struct RetroVariable {
    pub key: Option<String>,
    pub value: Option<String>
}

impl RetroVariable {
    pub unsafe fn from_void_ptr(ptr: *mut c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Report::msg("Null pointer"));
        }

        let data: bindings::retro_variable = *ptr.cast();  

        let key = {
            if data.key.is_null() {
                None
            } else {
                let c_key: &CStr = CStr::from_ptr(data.key);
                Some(c_key.to_str()?.to_owned())
            }
        };
        
        let value = {
            if data.value.is_null() {
                None
            } else {
                let c_value: &CStr = CStr::from_ptr(data.value);
                Some(c_value.to_str()?.to_owned())
            }
        };

        Ok(Self {
            key, value
        })
    }

    pub unsafe fn repopulate_void_ptr(&self, ptr: *mut c_void) -> Result<()> {
        if ptr.is_null() {
            return Err(Report::msg("Null pointer"));
        }

        let retro_variable_ptr: *mut bindings::retro_variable = ptr.cast(); 

        if let Some(string) = &self.value {
            let cstring = CString::new(string.as_str())?;
            (*retro_variable_ptr).value = cstring.into_raw();
        }

        Ok(())
    }
}



unsafe fn environ_cb_call(cb: EnvironmentCallback, cmd: u32, data: *mut c_void) -> bool {
    let env_cb_cmd = EnvironmentCallbackCmd::try_from(cmd);
    
    
    match env_cb_cmd {
        Ok(command) => {
            let env_cb_data = EnvironmentCallbackData::from_cmd_and_void_ptr(&command, data);
            match env_cb_data {
                Ok(mut cb_data) => {
                    let cb_result = cb(&command, &mut cb_data);

                    if !cb_result {
                        return false;
                    }

                    let result = cb_data.repopulate_void_ptr(data);

                    match result {
                        Ok(_) => true,
                        Err(_) => false
                    }
                },
                Err(_) => false
            }
        },
        Err(_) => false
    }
}


unsafe extern "C" fn environ_cb(cmd: u32, data: *mut c_void) -> bool {
    println!("extern cb, cmd: {}", cmd);

    let cb_lock_result = ENVIRON_CALLBACK_GLOBAL.read();
    match cb_lock_result {
        Err(_) => false,
        Ok(cb_lock) => {
            match *cb_lock {
                None => false,
                Some(cb) => {
                    environ_cb_call(cb, cmd, data)
                }
            }
        }

    }
}

pub fn set_environment_cb(cb: EnvironmentCallback) {
    {
        let mut lock = ENVIRON_CALLBACK_GLOBAL.write().unwrap();
        *lock = Some(cb);
    }

    unsafe {
        bindings::retro_set_environment(Some(environ_cb));
    }
}

