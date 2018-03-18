extern crate libloading as lib;

use std;
use std::ffi::{CString, OsStr};

use result::*;

pub enum CoreClrHostHandle {}
pub type CoreClrDomainId = u32;
pub type CoreClrString = *const std::os::raw::c_char;
pub type CoreClrDelegatePointer = *const ();

type CoreClrInitializeFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    exePath: CoreClrString,
    appDomainFriendlyName: CoreClrString,
    propertyCount: isize,
    propertyKeys: *const CoreClrString,
    propertyValues: *const CoreClrString,
    hostHandle: *mut *mut CoreClrHostHandle,
    domainId: *mut CoreClrDomainId) -> u32>;

type CoreClrExecuteAssemblyFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    hostHandle: *const CoreClrHostHandle,
    domainId: CoreClrDomainId,
    argc: isize,
    argv: *const CoreClrString,
    managedAssemblyPath: CoreClrString,
    exitCode: *mut u32) -> u32>;

type CoreClrCreateDelegateFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    hostHandle: *const CoreClrHostHandle,
    domainId: CoreClrDomainId,
    entryPointAssemblyName: CoreClrString,
    entryPointTypeName: CoreClrString,
    entryPointMethodName: CoreClrString,
    delegate: *mut *mut ()) -> u32>;

type CoreClrShutdownFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    hostHandle: *const CoreClrHostHandle, 
    domainId: CoreClrDomainId) -> u32>;

pub struct CoreClrInstance<'a> {    
    coreclr_execute_assembly: CoreClrExecuteAssemblyFn<'a>,
    coreclr_create_delegate: CoreClrCreateDelegateFn<'a>,
    coreclr_shutdown: CoreClrShutdownFn<'a>,
    clr_host_handle: *const CoreClrHostHandle,
    clr_domain_id: CoreClrDomainId,
}

impl<'a> CoreClrInstance<'a> {
    pub fn new(libclr: &'a lib::Library, app_paths: &str, app_ni_paths: &str, native_dll_search_dirs: &str) -> IronCoreResult<CoreClrInstance<'a>> {
        unsafe {
            let coreclr_initialize: CoreClrInitializeFn = libclr.get(b"coreclr_initialize")?;
            let coreclr_execute_assembly: CoreClrExecuteAssemblyFn = libclr.get(b"coreclr_execute_assembly")?;
            let coreclr_create_delegate: CoreClrCreateDelegateFn = libclr.get(b"coreclr_create_delegate")?;
            let coreclr_shutdown: CoreClrShutdownFn = libclr.get(b"coreclr_shutdown")?;

            let exe = std::env::current_exe()?;
            let exe_str = exe.to_str().ok_or(IronCoreError::InvalidExePath)?;
            let clr_exe_path = CString::new(exe_str)?;
            let clr_app_domain_friendly_name = CString::new("IronCore CLR Host")?;
            let clr_trusted_asms = get_trusted_assemblies()?;

            let property_keys: Vec<&str> = vec!["TRUSTED_PLATFORM_ASSEMBLIES", "APP_PATHS", "APP_NI_PATHS", "NATIVE_DLL_SEARCH_DIRECTORIES", "AppDomainCompatSwitch"];
            let (_clr_property_keys, clr_property_keys_ptr) = vec2cstring(property_keys)?;

            let property_values: Vec<&str> = vec![&clr_trusted_asms[..], app_paths, app_ni_paths, native_dll_search_dirs, "UseLatestBehaviorWhenTFMNotSpecified"];
            let (_clr_property_values, clr_property_values_ptr) = vec2cstring(property_values)?;

            let mut clr_host_handle: *mut CoreClrHostHandle = std::ptr::null_mut();
            let clr_host_handle_ptr: *mut *mut CoreClrHostHandle = &mut clr_host_handle;

            let mut clr_domain_id: CoreClrDomainId = 0u32;
            let clr_domain_id_ptr: *mut CoreClrDomainId = &mut clr_domain_id;

            let hr = HRESULT::from(coreclr_initialize(
                clr_exe_path.as_ptr(),
                clr_app_domain_friendly_name.as_ptr(),
                clr_property_keys_ptr.len() as isize,
                clr_property_keys_ptr.as_ptr(),
                clr_property_values_ptr.as_ptr(),
                clr_host_handle_ptr,
                clr_domain_id_ptr,
            ));
            hr.check()?;

            return Ok(CoreClrInstance { 
                coreclr_execute_assembly,
                coreclr_create_delegate,
                coreclr_shutdown,
                clr_host_handle,
                clr_domain_id
            });
        }
    }

    pub fn execute_assembly(&self, assembly: &str, args: Vec<&str>) -> IronCoreResult<u32> {
        unsafe {
            let clr_assembly = CString::new(assembly)?;

            let (_clr_args, clr_args_ptr) = vec2cstring(args)?;

            let mut clr_exit_code = 0u32;
            let clr_exit_code_ptr: *mut u32 = &mut clr_exit_code;

            let coreclr_execute_assembly = &self.coreclr_execute_assembly;
            let hr = HRESULT::from(coreclr_execute_assembly(
                self.clr_host_handle,
                self.clr_domain_id,
                clr_args_ptr.len() as isize,
                clr_args_ptr.as_ptr(),
                clr_assembly.as_ptr(),
                clr_exit_code_ptr
            ));
            hr.check()?;

            return Ok(clr_exit_code);
        }
    }

    pub fn create_delegate(&self, entry_point_assembly_name: &str, entry_point_type_name: &str, entry_point_method_name: &str) -> IronCoreResult<CoreClrDelegatePointer> {
        unsafe {
            let mut delegate = std::mem::uninitialized();

            let clr_entry_point_assembly_name = CString::new(entry_point_assembly_name)?;
            let clr_entry_point_type_name = CString::new(entry_point_type_name)?;
            let clr_entry_point_method_name = CString::new(entry_point_method_name)?;

            let coreclr_create_delegate = &self.coreclr_create_delegate;
            let hr = HRESULT::from(coreclr_create_delegate(
                self.clr_host_handle,
                self.clr_domain_id,
                clr_entry_point_assembly_name.as_ptr(),
                clr_entry_point_type_name.as_ptr(),
                clr_entry_point_method_name.as_ptr(),
                &mut delegate
            ));
            hr.check()?;

            return Ok(delegate);
        }
    }
}

impl<'a> Drop for CoreClrInstance<'a> {
    fn drop(&mut self) {
        unsafe {
            let coreclr_shutdown = &self.coreclr_shutdown;
            let _ = HRESULT::from(coreclr_shutdown(self.clr_host_handle, self.clr_domain_id));
        } 
    }
}

fn vec2cstring(strings: Vec<&str>) -> IronCoreResult<(Vec<CString>, Vec<CoreClrString>)> {
    let strings_result: std::result::Result<Vec<CString>, std::ffi::NulError> =
        strings
            .clone()
            .into_iter()
            .map(|x| CString::new(x))
            .collect();
    let cstrings = strings_result?;
    let cstrings_ptr: Vec<CoreClrString> =
        cstrings.iter().map(|x| x.as_ptr()).collect();

    return Ok((cstrings, cstrings_ptr));
}

fn get_runtime_dir() -> IronCoreResult<String> {
    // TODO: Add some actual probing logic
    let programfiles = std::env::var("PROGRAMFILES")?;
    let mut dir = String::new();
    dir.push_str(&programfiles);
    dir.push(std::path::MAIN_SEPARATOR);
    dir.push_str("dotnet");
    dir.push(std::path::MAIN_SEPARATOR);
    dir.push_str("shared");
    dir.push(std::path::MAIN_SEPARATOR);
    dir.push_str("Microsoft.NETCore.App");
    dir.push(std::path::MAIN_SEPARATOR);
    dir.push_str("2.0.6");

    return Ok(dir);
}

fn get_runtime_path() -> IronCoreResult<String> {
    let mut dll = String::clone(&get_runtime_dir()?);
    dll.push(std::path::MAIN_SEPARATOR);
    dll.push_str("CoreCLR.dll");

    return Ok(dll);
}

fn get_trusted_assemblies() -> IronCoreResult<String> {
    let mut result = String::new();

    let coreclr_dir = get_runtime_dir()?;
    let coreclr_files = std::fs::read_dir(coreclr_dir)?;
    for file in coreclr_files {
        let file = file?;
        let filepath = file.path();
        if let Some(fileext) = filepath.extension().and_then(OsStr::to_str) {
            if fileext == "dll" {
                if result.len() > 0 {
                    result.push_str(";");
                }
                if let Some(filepath_str) = filepath.to_str() {
                    result.push_str(filepath_str);
                }
            }
        }
    }

    return Ok(result);
}

pub fn load_coreclr_library() -> IronCoreResult<lib::Library> {
    let libclr_path = get_runtime_path()?;
    let libclr = lib::Library::new(std::ffi::OsString::from(libclr_path))?;
    return Ok(libclr);
}