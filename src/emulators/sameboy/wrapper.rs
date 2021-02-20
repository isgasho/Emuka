use std::{convert::TryFrom, ffi::{CStr, CString, c_void}, sync::RwLock};
use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use eyre::*;
use super::bindings;

#[repr(u32)]
#[derive(TryFromPrimitive, Debug)]
pub enum EnvironmentCallbackCmd {
    GetVariable = bindings::RETRO_ENVIRONMENT_GET_VARIABLE,

    GetSystemDirectory = bindings::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
    GetSaveDirectory = bindings::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
}

#[derive(Debug)]
pub enum EnvironmentCallbackData {
    RetroVariable(RetroVariable),
    StringWrapper(StringWrapper)
}

impl EnvironmentCallbackData {
    pub unsafe fn from_cmd_and_void_ptr(cmd: &EnvironmentCallbackCmd, data: *mut c_void) -> Result<Self> {
        use EnvironmentCallbackCmd::*;

        Ok(match cmd {
            GetVariable => Self::RetroVariable(RetroVariable::from_void_ptr(data)?),
            GetSystemDirectory => Self::StringWrapper(StringWrapper::from_void_ptr(data)?),
            GetSaveDirectory => Self::StringWrapper(StringWrapper::from_void_ptr(data)?)
        })
    }

    pub unsafe fn repopulate_void_ptr(&self, data: *mut c_void) -> Result<()> {
        use EnvironmentCallbackData::*;
        
        Ok(match self {
            RetroVariable(retro_variable) => {
                retro_variable.repopulate_void_ptr(data)?;
            },
            StringWrapper(wrapper) => {
                println!("{:?}", data);
                wrapper.repopulate_void_ptr(data)?;
                println!("{:?}", data);
            }
            
        })
    }
}

pub type EnvironmentCallback = fn(cmd: &EnvironmentCallbackCmd, data: &mut EnvironmentCallbackData) -> bool;

lazy_static! {
    static ref ENVIRON_CALLBACK_GLOBAL: RwLock<Option<EnvironmentCallback>> = RwLock::new(None);
}

unsafe fn interpret_cstring(ptr: *const i8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let cstr: &CStr = CStr::from_ptr(ptr);
    let ref_str = cstr.to_str();
    match ref_str {
        Ok(ref_str) => Some(ref_str.to_owned()),
        Err(err) => {
            println!("{:?}", err);
            None
        } 
    }
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

        let key = interpret_cstring(data.key);
        let value = interpret_cstring(data.value);

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
        } else {
            (*retro_variable_ptr).value = std::ptr::null();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct StringWrapper {
    pub inner: Option<String>
}

impl StringWrapper {
    pub unsafe fn from_void_ptr(ptr: *mut c_void) -> Result<Self> {       
        let cstring_ptr: *const i8 = ptr.cast();
        let inner = interpret_cstring(cstring_ptr);

        Ok(Self {inner})
    }

    pub unsafe fn repopulate_void_ptr(&self, ptr: *mut c_void) -> Result<()> {
        let data: *mut *const c_void = ptr.cast(); 
        
        if let Some(string) = &self.inner {
            let cstring = CString::new(string.as_str())?;
            *data = cstring.into_raw().cast();
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
                        Err(err) => {
                            println!("{:?}", err);
                            false
                        }
                    }
                },
                Err(err) => {
                    println!("{:?}", err);
                    false
                }
            }
        },
        Err(err) => {
            println!("{:?}", err);
            false
        }
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

