use std::{convert::TryFrom, ffi::{CStr, CString, c_void}, os::raw::c_uint, panic::{AssertUnwindSafe, catch_unwind}, path::{self, Path}, sync::RwLock};
use bindings::{RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B, RETRO_DEVICE_ID_JOYPAD_START, emuka_save_battery, retro_set_input_poll};
use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use eyre::*;
use super::bindings::{self, size_t};

#[repr(u32)]
#[derive(TryFromPrimitive, Debug)]
pub enum EnvironmentCallbackCmd {
    GetVariable = bindings::RETRO_ENVIRONMENT_GET_VARIABLE,

    GetSystemDirectory = bindings::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
    GetSaveDirectory = bindings::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,

    SetPixelFormat = bindings::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
}

#[derive(Debug)]
pub enum EnvironmentCallbackData {
    RetroVariable(RetroVariable),
    StringWrapper(StringWrapper),
    IntWrapper(IntWrapper)
}

impl EnvironmentCallbackData {
    pub unsafe fn from_cmd_and_void_ptr(cmd: &EnvironmentCallbackCmd, data: *mut c_void) -> Result<Self> {
        use EnvironmentCallbackCmd::*;

        Ok(match cmd {
            GetVariable => Self::RetroVariable(RetroVariable::from_void_ptr(data)?),
            GetSystemDirectory => Self::StringWrapper(StringWrapper::from_void_ptr(data)?),
            GetSaveDirectory => Self::StringWrapper(StringWrapper::from_void_ptr(data)?),
            SetPixelFormat => Self::IntWrapper(IntWrapper::from_void_ptr(data)?),
        })
    }

    pub unsafe fn repopulate_void_ptr(&self, data: *mut c_void) -> Result<()> {
        use EnvironmentCallbackData::*;
        
        Ok(match self {
            RetroVariable(retro_variable) => {
                retro_variable.repopulate_void_ptr(data)?;
            },
            StringWrapper(wrapper) => {
                wrapper.repopulate_void_ptr(data)?;
            },
            IntWrapper(wrapper) => {
                wrapper.repopulate_void_ptr(data)?
            }
        })
    }
}
#[repr(u32)]
#[derive(TryFromPrimitive, Debug)]
pub enum JoypadInput {
    A = bindings::RETRO_DEVICE_ID_JOYPAD_A,
    B = bindings::RETRO_DEVICE_ID_JOYPAD_B,

    START = bindings::RETRO_DEVICE_ID_JOYPAD_START,
    SELECT = bindings::RETRO_DEVICE_ID_JOYPAD_SELECT,

    UP = bindings::RETRO_DEVICE_ID_JOYPAD_UP,
    DOWN = bindings::RETRO_DEVICE_ID_JOYPAD_DOWN,
    LEFT = bindings::RETRO_DEVICE_ID_JOYPAD_LEFT,
    RIGHT = bindings::RETRO_DEVICE_ID_JOYPAD_RIGHT,
}

pub type EnvironmentCallback = fn(cmd: &EnvironmentCallbackCmd, data: &mut EnvironmentCallbackData) -> bool;
pub type InputPollCallback = fn();
pub type InputStateCallback = fn(JoypadInput) -> i16;
pub type AudioSampleCallback = fn(i16, i16);
pub type VideoRefreshCallback = fn(&[u32], u32, u32, u64);


lazy_static! {
    static ref ENVIRON_CALLBACK_GLOBAL: RwLock<Option<EnvironmentCallback>> = RwLock::new(None);
    static ref INPUT_POLL_CALLBACK_GLOBAL: RwLock<Option<InputPollCallback>> = RwLock::new(None);
    static ref INPUT_STATE_CALLBACK_GLOBAL: RwLock<Option<InputStateCallback>> = RwLock::new(None);
    static ref AUDIO_SAMPLE_CALLBACK_GLOBAL: RwLock<Option<AudioSampleCallback>> = RwLock::new(None);
    static ref VIDEO_REFRESH_CALLBACK_GLOBAL: RwLock<Option<VideoRefreshCallback>> = RwLock::new(None);
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
        if ptr.is_null() {
            return Err(Report::msg("Null pointer"));
        }
        
        let cstring_ptr: *const i8 = ptr.cast();
        let inner = interpret_cstring(cstring_ptr);

        Ok(Self {inner})
    }

    pub unsafe fn repopulate_void_ptr(&self, ptr: *mut c_void) -> Result<()> {
        if ptr.is_null() {
            return Err(Report::msg("Null pointer"));
        }
        
        let data: *mut *const c_void = ptr.cast(); 
        
        if let Some(string) = &self.inner {
            let cstring = CString::new(string.as_str())?;
            *data = cstring.into_raw().cast();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct IntWrapper {
    pub inner: u32
}

impl IntWrapper {
    pub unsafe fn from_void_ptr(ptr: *mut c_void) -> Result<Self> {
        if ptr.is_null() {
            return Err(Report::msg("Null pointer"));
        }

        let int_ptr: *const u32 = ptr.cast();
        Ok(Self { inner: *int_ptr })
    }

    pub unsafe fn repopulate_void_ptr(&self, ptr: *mut c_void) -> Result<()> {
        if ptr.is_null() {
            return Err(Report::msg("Null pointer"));
        }

        let data: *mut u32 = ptr.cast();
        *data = self.inner;
        
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
                    let cb_result = catch_unwind(
                        AssertUnwindSafe(|| cb(&command, &mut cb_data))
                    );

                    if let Err(err) = cb_result {
                        println!("{:?}", err);
                        return false;
                    } 

                    if !cb_result.unwrap() {
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
            // println!("{:?}", err);
            false
        }
    }
}


unsafe extern "C" fn environ_cb(cmd: u32, data: *mut c_void) -> bool {
    // println!("extern cb, cmd: {}", cmd);

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

#[derive(Debug)]
pub struct GameInfo {
    pub path: String
}

pub fn init() {
    unsafe {
        bindings::retro_init();
    }
}

pub fn load_game(game_info: &GameInfo) -> bool {
    let cstring = CString::new(game_info.path.as_str()).unwrap();
    let cstr_ptr = cstring.into_raw();

    let retro_game_info = bindings::retro_game_info {
        path: cstr_ptr,
        data: std::ptr::null(),
        size: 0,
        meta: std::ptr::null()
    };

    unsafe {
        bindings::retro_load_game(&retro_game_info)
    }
}

pub fn run_frame() {
    unsafe {
        bindings::retro_run();
    }
}

unsafe extern "C" fn input_poll_cb() {
    let cb_lock_result = INPUT_POLL_CALLBACK_GLOBAL.read();
    match cb_lock_result {
        Err(_) => (),
        Ok(cb_lock) => {
            match *cb_lock {
                None => (),
                Some(cb) => {
                    cb()
                }
            }
        }
    }
}

pub fn set_input_poll_cb(cb: InputPollCallback) {
    {
        let mut lock = INPUT_POLL_CALLBACK_GLOBAL.write().unwrap();
        *lock = Some(cb);
    }

    unsafe {
        bindings::retro_set_input_poll(Some(input_poll_cb));
    }
}

fn input_state_call(cb: InputStateCallback, input: u32) -> i16 {
    match JoypadInput::try_from(input) {
        Ok(input) => {
            let cb_result = catch_unwind(|| cb(input));

            match cb_result {
                Ok(result) => result,
                Err(err) => {
                    println!("{:?}", err);
                    0
                }
            }
        },
        Err(err) => {
            // println!("{:?}", err);
            0
        }
    }
}

unsafe extern "C" fn input_state_cb(_port: u32, _device: u32, _index: u32, id: u32) -> i16 {
    let cb_lock_result = INPUT_STATE_CALLBACK_GLOBAL.read();
    match cb_lock_result {
        Err(_) => 0i16,
        Ok(cb_lock) => {
            match *cb_lock {
                None => 0i16,
                Some(cb) => {
                    input_state_call(cb, id)
                }
            }
        }
    }
}


pub fn set_input_state_cb(cb: InputStateCallback) {
    {
        let mut lock = INPUT_STATE_CALLBACK_GLOBAL.write().unwrap();
        *lock = Some(cb);
    }

    unsafe {
        bindings::retro_set_input_state(Some(input_state_cb));
    }
}


fn audio_sample_call(cb: AudioSampleCallback, left: i16, right: i16) {
    let cb_result = catch_unwind(|| cb(left, right));

    match cb_result {
        Ok(result) => result,
        Err(err) => {
            println!("{:?}", err);
        }
    }
}

unsafe extern "C" fn audio_sample_cb(left: i16, right: i16) {
    let cb_lock_result = AUDIO_SAMPLE_CALLBACK_GLOBAL.read();
    match cb_lock_result {
        Err(_) => (),
        Ok(cb_lock) => {
            match *cb_lock {
                None => (),
                Some(cb) => {
                    audio_sample_call(cb, left, right)
                }
            }
        }
    }
}


pub fn set_audio_sample_cb(cb: AudioSampleCallback) {
    {
        let mut lock = AUDIO_SAMPLE_CALLBACK_GLOBAL.write().unwrap();
        *lock = Some(cb);
    }

    unsafe {
        bindings::retro_set_audio_sample(Some(audio_sample_cb));
    }
}

unsafe fn video_refresh_call(cb: VideoRefreshCallback, data: *const c_void, width: c_uint, height: c_uint, pitch: size_t) {
    if data.is_null() {
        println!("Null pointer on video refresh");
        return;
    }

    let data: &[u32] = std::slice::from_raw_parts(data.cast(), (width * height) as usize); 

    let cb_result = catch_unwind(|| cb(data, width, height, pitch));

    match cb_result {
        Ok(result) => result,
        Err(err) => {
            println!("{:?}", err);
        }
    }
}

unsafe extern "C" fn video_refresh_cb(data: *const c_void, width: c_uint, height: c_uint, pitch: size_t) {
    let cb_lock_result = VIDEO_REFRESH_CALLBACK_GLOBAL.read();
    match cb_lock_result {
        Err(_) => (),
        Ok(cb_lock) => {
            match *cb_lock {
                None => (),
                Some(cb) => {
                    video_refresh_call(cb, data, width, height, pitch)
                }
            }
        }
    }
}


pub fn set_video_refresh_cb(cb: VideoRefreshCallback) {
    {
        let mut lock = VIDEO_REFRESH_CALLBACK_GLOBAL.write().unwrap();
        *lock = Some(cb);
    }

    unsafe {
        bindings::retro_set_video_refresh(Some(video_refresh_cb));
    }
}

pub fn unload_game() {
    unsafe {
        bindings::retro_unload_game();
    }
}

pub fn deinit() {
    unsafe {
        bindings::retro_deinit();
    }
}

pub fn save <P: AsRef<Path>> (path: P) {
    let cstring = CString::new(path.as_ref().to_str().unwrap()).unwrap();
    let cstr_ptr = cstring.into_raw();

    unsafe {
        bindings::emuka_save_battery(cstr_ptr);
    }

    let cstr = unsafe { CString::from_raw(cstr_ptr) };
}