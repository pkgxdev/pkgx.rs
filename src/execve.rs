use std::ffi::CString;
use std::ptr;
use std::{collections::HashMap, error::Error};

pub fn execve(
    cmd: String,
    mut args: Vec<String>,
    env: HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    // Convert the command to a CString
    let c_command = CString::new(cmd.clone())
        .map_err(|e| format!("Failed to convert command to CString: {}", e))?;

    // execve expects the command to be the first argument (yes, as well)
    args.insert(0, cmd);

    // Convert the arguments to CStrings and collect them into a Vec
    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| {
            CString::new(arg.clone())
                .map_err(|e| format!("Failed to convert argument to CString: {}", e))
        })
        .collect::<Result<_, _>>()?;
    let mut c_args_ptrs: Vec<*const i8> = c_args.iter().map(|arg| arg.as_ptr()).collect();
    c_args_ptrs.push(ptr::null()); // Null-terminate the arguments array

    // Convert the environment to a Vec of `KEY=VALUE` strings
    let env_vars: Vec<String> = env
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();

    // Convert the environment variables to CStrings and collect them into a Vec
    let c_env: Vec<CString> = env_vars
        .iter()
        .map(|env| {
            CString::new(env.clone())
                .map_err(|e| format!("Failed to convert environment variable to CString: {}", e))
        })
        .collect::<Result<_, _>>()?;
    let mut c_env_ptrs: Vec<*const i8> = c_env.iter().map(|env| env.as_ptr()).collect();
    c_env_ptrs.push(ptr::null()); // Null-terminate the environment array

    unsafe {
        // Replace the process with the new command, arguments, and environment
        if libc::execve(
            c_command.as_ptr(),
            c_args_ptrs.as_ptr(),
            c_env_ptrs.as_ptr(),
        ) == -1
        {
            let err = *libc::__error();
            return Err(format!("execve failed with errno: {}", err).into());
        }
    }

    Ok(())
}
