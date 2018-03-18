pub mod result;
pub mod clr;

use result::{IronCoreResult, IronCoreError};

fn get_exe_path() -> IronCoreResult<String> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.as_path().parent().ok_or(IronCoreError::InvalidExePath)?;
    let exe_dir_path = exe_dir.to_str().ok_or(IronCoreError::InvalidExePath)?;
    return Ok(String::from(exe_dir_path));
}

fn main() {
    let libclr = clr::load_coreclr_library().unwrap();
    {
        let coreclr_app_paths = &get_exe_path().unwrap();
        let coreclr_ni_app_paths = coreclr_app_paths;
        let coreclr_dll_native_search_dirs = coreclr_app_paths;

        let coreclr = clr::CoreClrInstance::new(&libclr, coreclr_app_paths, coreclr_ni_app_paths, coreclr_dll_native_search_dirs).unwrap();

        unsafe {
            let delegate_ptr = coreclr.create_delegate("ironcore-example", "IronCore.Example.Scripts", "Main").unwrap();
            let delegate = std::mem::transmute::<clr::CoreClrDelegatePointer, extern "system" fn() -> ()>(delegate_ptr);
            delegate();
        }
    }
    std::mem::forget(libclr);
}
