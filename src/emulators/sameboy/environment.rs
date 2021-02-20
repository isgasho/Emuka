use super::wrapper::*;

pub fn environment_callback(cmd: &EnvironmentCallbackCmd, data: &mut EnvironmentCallbackData) -> bool {
    use EnvironmentCallbackCmd::*;
    println!("env cb called; cmd: {:?}; data: {:?}", cmd, data);

    match cmd {
        GetVariable => get_variable(data),
        GetSystemDirectory => get_system_directory(data),
        GetSaveDirectory => get_system_directory(data)
    }
}


fn get_variable(data: &mut EnvironmentCallbackData) -> bool {
    match data {
        EnvironmentCallbackData::RetroVariable(retro_variable) => {
            if let Some(key) = &retro_variable.key {
                if key == "sameboy_model" {
                    retro_variable.value = Some(String::from("Game Boy Color"));
                    return true;
                }
            }
        },
        _ => return false 
    };
    return false;
}


fn get_system_directory(data: &mut EnvironmentCallbackData) -> bool {
    match data {
        EnvironmentCallbackData::StringWrapper(wrapper) => {
            wrapper.inner = Some(String::from("./game"));
            return true;
        }
        _ => return false
    }
}
