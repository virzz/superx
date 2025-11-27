use std::ffi::{CStr, CString};
use std::ptr;

use security_framework_sys::authorization::*;

// vproc_priv.h 中定义的枚举类型
// vproc_gsk_t 枚举值
const VPROC_GSK_MGR_NAME: i32 = 6;
// VPROCMGR_SESSION_AQUA 字符串常量
// 这是 launchd 中定义的常量，值为 "Aqua"
const VPROCMGR_SESSION_AQUA: &[u8] = b"Aqua\0";

// vproc_err_t 类型（通常是整数）
pub type VprocErr = libc::c_int;
// vproc_t 类型（通常是指针）
pub type Vproc = *const libc::c_void;

// macOS vproc 相关函数
unsafe extern "C" {
    // vproc_priv.h 中的函数
    // vproc_err_t vproc_swap_string(vproc_t vp, vproc_gsk_t key, const char *instr, char **outstr);
    pub fn vproc_swap_string(
        vp: Vproc,
        key: libc::c_int, // vproc_gsk_t 实际上是整数类型
        instr: *const libc::c_char,
        outstr: *mut *mut libc::c_char,
    ) -> VprocErr;
}

// 密码输入相关函数
unsafe extern "C" {
    pub fn readpassphrase(
        prompt: *const libc::c_char,
        buf: *mut libc::c_char,
        bufsiz: libc::size_t,
        flags: libc::c_int,
    ) -> *mut libc::c_char;
}

pub const RPP_ECHO_OFF: libc::c_int = 0x02;

/// 检查是否在 Aqua session 中运行
pub fn check_aqua_session() -> bool {
    unsafe {
        let mut manager: *mut libc::c_char = ptr::null_mut();
        // vproc_swap_string 返回 0 表示成功，非 0 表示错误
        if vproc_swap_string(ptr::null(), VPROC_GSK_MGR_NAME, ptr::null(), &mut manager) != 0
            || manager.is_null()
        {
            return false;
        }
        // 与 VPROCMGR_SESSION_AQUA 字符串比较
        let aqua = VPROCMGR_SESSION_AQUA.as_ptr() as *const libc::c_char;
        let is_aqua = libc::strcmp(manager, aqua) == 0;
        libc::free(manager as *mut libc::c_void);
        is_aqua
    }
}

/// 使用 macOS Authorization API 进行认证（支持 Touch ID 或密码）
pub fn authenticate() -> bool {
    unsafe {
        let mut authorization_ref: AuthorizationRef = ptr::null_mut();
        // 创建授权引用
        let status = AuthorizationCreate(
            ptr::null(),
            ptr::null(),
            kAuthorizationFlagDefaults,
            &mut authorization_ref,
        );
        if status != errAuthorizationSuccess {
            return false;
        }
        // 创建授权项：验证当前用户权限
        let right_name = CString::new("mozhu").unwrap();
        let mut item = AuthorizationItem {
            name: right_name.as_ptr(),
            valueLength: 0,
            value: ptr::null_mut(),
            flags: 0,
        };
        let rights = AuthorizationRights {
            count: 1,
            items: &mut item as *mut AuthorizationItem,
        };
        // 请求授权，这会触发 Touch ID 或密码认证对话框来验证当前用户身份
        let flags = kAuthorizationFlagDefaults
            | kAuthorizationFlagInteractionAllowed
            | kAuthorizationFlagExtendRights;
        let mut authorized_rights: *mut AuthorizationRights = ptr::null_mut();
        let empty_env = AuthorizationEnvironment {
            count: 0,
            items: ptr::null_mut(),
        };
        let status = AuthorizationCopyRights(
            authorization_ref,
            &rights,
            &empty_env,
            flags,
            &mut authorized_rights,
        );
        // 清理授权项集
        if !authorized_rights.is_null() {
            AuthorizationFreeItemSet(authorized_rights);
        }
        // 释放授权引用
        AuthorizationFree(authorization_ref, kAuthorizationFlagDefaults);
        status == errAuthorizationSuccess
    }
}

/// 从终端读取密码（隐藏输入）
pub fn read_password(prompt: &str) -> Option<String> {
    unsafe {
        let prompt_cstr = match CString::new(prompt) {
            Ok(s) => s,
            Err(_) => {
                return None;
            }
        };
        let mut buffer = vec![0u8; 1024];
        let result = readpassphrase(
            prompt_cstr.as_ptr(),
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len() - 1,
            RPP_ECHO_OFF,
        );
        if result.is_null() {
            return None;
        }
        let c_str = CStr::from_ptr(buffer.as_ptr() as *const libc::c_char);
        match c_str.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => None,
        }
    }
}

/// 使用命令行密码进行认证
/// 使用 macOS Authorization API 通过环境变量传递密码
pub fn authenticate_with_password(password: &str) -> bool {
    unsafe {
        let mut authorization_ref: AuthorizationRef = ptr::null_mut();
        // 创建授权引用
        let status = AuthorizationCreate(
            ptr::null(),
            ptr::null(),
            kAuthorizationFlagDefaults,
            &mut authorization_ref,
        );
        if status != errAuthorizationSuccess {
            return false;
        }
        // 准备密码字符串
        let password_cstr = match CString::new(password) {
            Ok(s) => s,
            Err(_) => {
                AuthorizationFree(authorization_ref, kAuthorizationFlagDefaults);
                return false;
            }
        };
        // 创建包含密码的环境变量
        // 注意：密码环境变量的名称必须是 "password"（小写）
        let password_name = CString::new("password").unwrap();
        // 获取密码字节（不包括 null 终止符）
        let password_bytes = password_cstr.as_bytes();
        let password_len = password_bytes.len();
        // 分配内存来存储密码（Authorization API 会复制这个值）
        let password_ptr = libc::malloc(password_len) as *mut libc::c_void;
        if password_ptr.is_null() {
            AuthorizationFree(authorization_ref, kAuthorizationFlagDefaults);
            return false;
        }
        libc::memcpy(
            password_ptr,
            password_bytes.as_ptr() as *const libc::c_void,
            password_len,
        );
        let mut env_item = AuthorizationItem {
            name: password_name.as_ptr(),
            valueLength: password_len,
            value: password_ptr,
            flags: 0,
        };
        let environment = AuthorizationEnvironment {
            count: 1,
            items: &mut env_item as *mut AuthorizationItem,
        };
        // 创建授权项：验证当前用户权限
        let right_name = CString::new("allow").unwrap();
        let mut item = AuthorizationItem {
            name: right_name.as_ptr(),
            valueLength: 0,
            value: ptr::null_mut(),
            flags: 0,
        };
        let rights = AuthorizationRights {
            count: 1,
            items: &mut item as *mut AuthorizationItem,
        };
        // 请求授权，使用提供的密码验证当前用户身份
        let flags = kAuthorizationFlagDefaults | kAuthorizationFlagExtendRights;
        let mut authorized_rights: *mut AuthorizationRights = ptr::null_mut();
        let status = AuthorizationCopyRights(
            authorization_ref,
            &rights,
            &environment,
            flags,
            &mut authorized_rights,
        );
        // 清理密码内存（在验证完成后立即清除）
        libc::memset(password_ptr, 0, password_len);
        libc::free(password_ptr);
        // 清理授权项集
        if !authorized_rights.is_null() {
            AuthorizationFreeItemSet(authorized_rights);
        }
        // 释放授权引用
        AuthorizationFree(authorization_ref, kAuthorizationFlagDefaults);
        status == errAuthorizationSuccess
    }
}

pub fn check_permission() -> bool {
    // 首先尝试图形界面认证（支持 Touch ID 或密码对话框）
    if check_aqua_session() && authenticate() {
        return true;
    }
    // 如果图形界面认证失败，使用命令行密码输入
    match read_password("Password: ") {
        Some(pwd) => authenticate_with_password(&pwd),
        None => false,
    }
}
