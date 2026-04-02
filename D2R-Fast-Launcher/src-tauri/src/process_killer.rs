use std::mem::size_of;
use windows::Win32::Foundation::{CloseHandle, DuplicateHandle, DUPLICATE_CLOSE_SOURCE, HANDLE, STATUS_SUCCESS, STATUS_INFO_LENGTH_MISMATCH};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcess, PROCESS_DUP_HANDLE, PROCESS_QUERY_INFORMATION};
use ntapi::ntexapi::{
    NtQuerySystemInformation, SystemHandleInformation, SYSTEM_HANDLE_INFORMATION,
};
use ntapi::ntobapi::{
    NtQueryObject, ObjectNameInformation, ObjectTypeInformation, OBJECT_TYPE_INFORMATION,
};
use winapi::shared::ntdef::UNICODE_STRING;
use windows::Win32::System::Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE};

pub fn kill_d2r_mutexes_for_pid(target_pid: u32) -> Result<u32, String> {
    unsafe {
        let mut buffer_size: u32 = 0x10000;
        let mut buffer = VirtualAlloc(None, buffer_size as usize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        
        if buffer.is_null() {
            return Err("Failed to allocate memory for handle query".into());
        }

        let mut return_length: u32 = 0;
        let mut status = NtQuerySystemInformation(
            SystemHandleInformation,
            buffer as *mut _,
            buffer_size,
            &mut return_length,
        );

        while status == STATUS_INFO_LENGTH_MISMATCH.0 as i32 {
            let _ = VirtualFree(buffer, 0, MEM_RELEASE);
            buffer_size = return_length;
            buffer = VirtualAlloc(None, buffer_size as usize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
            if buffer.is_null() {
                return Err("Failed to re-allocate memory".into());
            }
            status = NtQuerySystemInformation(
                SystemHandleInformation,
                buffer as *mut _,
                buffer_size,
                &mut return_length,
            );
        }

        if status != STATUS_SUCCESS.0 as i32 {
            let _ = VirtualFree(buffer, 0, MEM_RELEASE);
            return Err(format!("NtQuerySystemInformation failed with status: {:#x}", status));
        }

        let handle_info = &*(buffer as *const SYSTEM_HANDLE_INFORMATION);
        let handle_count = handle_info.NumberOfHandles as usize;
        
        let handles_ptr = (buffer as usize + size_of::<usize>()) as *const ntapi::ntexapi::SYSTEM_HANDLE_TABLE_ENTRY_INFO;
        let handles = std::slice::from_raw_parts(handles_ptr, handle_count);

        let mut killed_count = 0;

        let process_handle = match OpenProcess(
            PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION,
            false,
            target_pid,
        ) {
            Ok(h) => h,
            Err(_) => {
                let _ = VirtualFree(buffer, 0, MEM_RELEASE);
                return Ok(0); // Cannot open process
            }
        };

        let current_process = GetCurrentProcess();

        for handle_entry in handles {
            if handle_entry.UniqueProcessId as u32 == target_pid {
                let mut dup_handle: HANDLE = HANDLE(0);
                let success = DuplicateHandle(
                    process_handle,
                    HANDLE(handle_entry.HandleValue as isize),
                    current_process,
                    &mut dup_handle,
                    0,
                    false,
                    windows::Win32::Foundation::DUPLICATE_SAME_ACCESS,
                );

                if success.is_ok() && !dup_handle.is_invalid() {
                    // 1. Get Object Type
                    let mut type_buf = vec![0u8; 1024];
                    let mut ret_len = 0;
                    let t_status = NtQueryObject(
                        dup_handle.0 as *mut _,
                        ObjectTypeInformation,
                        type_buf.as_mut_ptr() as *mut _,
                        type_buf.len() as u32,
                        &mut ret_len,
                    );

                    if t_status == STATUS_SUCCESS.0 as i32 {
                        let type_info = &*(type_buf.as_ptr() as *const OBJECT_TYPE_INFORMATION);
                        let type_name_slice = std::slice::from_raw_parts(
                            type_info.TypeName.Buffer,
                            (type_info.TypeName.Length / 2) as usize,
                        );
                        let type_name = String::from_utf16_lossy(type_name_slice);

                        // D2R uses an "Event" object for its Mutex. We only query name for Event to avoid deadlocks.
                        if type_name == "Event" || type_name == "Mutant" || type_name == "Section" {
                            let mut name_buf = vec![0u8; 2048];
                            let mut n_ret_len = 0;
                            let n_status = NtQueryObject(
                                dup_handle.0 as *mut _,
                                ObjectNameInformation,
                                name_buf.as_mut_ptr() as *mut _,
                                name_buf.len() as u32,
                                &mut n_ret_len,
                            );

                            if n_status == STATUS_SUCCESS.0 as i32 {
                                // First bytes are UNICODE_STRING
                                let uni_str = &*(name_buf.as_ptr() as *const UNICODE_STRING);
                                if !uni_str.Buffer.is_null() && uni_str.Length > 0 {
                                    let name_slice = std::slice::from_raw_parts(
                                        uni_str.Buffer,
                                        (uni_str.Length / 2) as usize,
                                    );
                                    let obj_name = String::from_utf16_lossy(name_slice);

                                    if obj_name.contains("DiabloII Check For Other Instances") {
                                        // Found the mutex! Force close it from the target process.
                                        let mut dummy: HANDLE = HANDLE(0);
                                        let _ = DuplicateHandle(
                                            process_handle,
                                            HANDLE(handle_entry.HandleValue as isize),
                                            current_process,
                                            &mut dummy,
                                            0,
                                            false,
                                            DUPLICATE_CLOSE_SOURCE,
                                        );
                                        if !dummy.is_invalid() {
                                            let _ = CloseHandle(dummy);
                                        }
                                        killed_count += 1;
                                    }
                                }
                            }
                        }
                    }
                    let _ = CloseHandle(dup_handle);
                }
            }
        }

        let _ = CloseHandle(process_handle);
        let _ = VirtualFree(buffer, 0, MEM_RELEASE);
        Ok(killed_count)
    }
}

pub fn kill_mutex_by_handle(target_pid: u32, handle_value: usize) -> Result<(), String> {
    unsafe {
        let process_handle = OpenProcess(
            PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION,
            false,
            target_pid,
        ).map_err(|e| format!("Failed to open process: {}", e))?;

        let current_process = GetCurrentProcess();
        let mut target_handle_copy: HANDLE = HANDLE(0);

        let success = DuplicateHandle(
            process_handle,
            HANDLE(handle_value as isize),
            current_process,
            &mut target_handle_copy,
            0,
            false,
            DUPLICATE_CLOSE_SOURCE,
        );

        let _ = CloseHandle(process_handle);

        if success.is_ok() {
            let _ = CloseHandle(target_handle_copy);
            Ok(())
        } else {
            Err("Failed to duplicate and close target handle".into())
        }
    }
}
