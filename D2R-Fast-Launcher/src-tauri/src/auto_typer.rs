use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, KEYBDINPUT, VK_RETURN, VK_TAB
};

fn send_key(vk: u16) {
    unsafe {
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk),
                    wScan: 0,
                    dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        // 按下
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        
        // 释放
        input.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

fn type_string(s: &str) {
    for c in s.chars() {
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                    wScan: c as u16,
                    dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            // 按下
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            
            // 释放
            input.Anonymous.ki.dwFlags = windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn type_credentials(username: &str, password: &str) -> Result<(), String> {
    // 假设游戏窗口已经处于前台并获得了焦点
    // 实际生产环境可能需要调用 SetForegroundWindow 确保焦点
    
    // 输入账号
    type_string(username);
    thread::sleep(Duration::from_millis(100));
    
    // 按 Tab 切换到密码框
    send_key(VK_TAB.0);
    thread::sleep(Duration::from_millis(100));
    
    // 输入密码
    type_string(password);
    thread::sleep(Duration::from_millis(100));
    
    // 按回车登录
    send_key(VK_RETURN.0);
    
    Ok(())
}
