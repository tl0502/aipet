// I.2 CryptoService — Windows DPAPI 封装
//
// 用途:secrets 表存 LLM provider API key 的密文(B.1 LLMProvider 调用前 unprotect)。
// 绑定:ciphertext 与当前登录用户 + 机器绑定,跨用户/跨机器无法解密 — 这是 DPAPI 的安全模型,
//       与 macOS Keychain / Linux libsecret 在不同 OS 上等价。MVP 期 Windows-only,详见
//       架构 v1.1 §4 secrets 表。
// 防 UI:CRYPTPROTECT_UI_FORBIDDEN(0x1)— 任何场景下 DPAPI 都不能弹密码 UI,弹了立即报错。
//       这是隐私边界 #4 的硬性要求:加密对用户无感。
//
// 测试通过路径:`cargo test --manifest-path src-tauri/Cargo.toml crypto`
// 注意:测试需要在 Windows 用户登录态运行(DPAPI 依赖用户 token)。

use thiserror::Error;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("DPAPI protect failed: {0}")]
    ProtectFailed(#[source] windows::core::Error),

    #[error("DPAPI unprotect failed: {0}")]
    UnprotectFailed(#[source] windows::core::Error),
}

/// 用 DPAPI 加密一段数据。返回的 ciphertext 已包含 DPAPI metadata,直接存数据库即可。
///
/// `plaintext` 可为空切片;DPAPI 仍会返回非空 ciphertext(只含 metadata)。
pub fn protect(plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: plaintext.len() as u32,
        pbData: plaintext.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptProtectData(
            &in_blob,
            PCWSTR::null(),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
        .map_err(CryptoError::ProtectFailed)?;

        let result = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
        Ok(result)
    }
}

/// 用 DPAPI 解密 `protect` 产出的 ciphertext。
///
/// 失败场景:ciphertext 非 DPAPI 输出 / 跨用户跨机器无法解密 / Windows DPAPI key 损坏。
pub fn unprotect(ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: ciphertext.len() as u32,
        pbData: ciphertext.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptUnprotectData(
            &in_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
        .map_err(CryptoError::UnprotectFailed)?;

        let result = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_typical_api_key() {
        let plaintext = b"sk-test-1234567890abcdef";
        let ciphertext = protect(plaintext).expect("protect should succeed under user login");
        assert_ne!(
            ciphertext.as_slice(),
            plaintext,
            "ciphertext must differ from plaintext"
        );
        let recovered = unprotect(&ciphertext).expect("unprotect should round-trip");
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn empty_input_round_trips() {
        // 空输入也应能成功:DPAPI 仍写 metadata,unprotect 还原为空。
        let plaintext: &[u8] = b"";
        let ciphertext = protect(plaintext).expect("protect empty");
        assert!(
            !ciphertext.is_empty(),
            "DPAPI metadata makes ciphertext non-empty even for empty input"
        );
        let recovered = unprotect(&ciphertext).expect("unprotect empty");
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn binary_safety_with_embedded_nulls() {
        // 二进制安全性:含 0x00 / 0xFF 也必须还原。
        let plaintext: &[u8] = &[0x00, 0xFF, 0x01, 0x00, 0xAB, 0xCD, 0x00, 0x7F];
        let ciphertext = protect(plaintext).expect("protect binary");
        let recovered = unprotect(&ciphertext).expect("unprotect binary");
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn invalid_ciphertext_returns_error() {
        // 非 DPAPI 产物必须 unprotect 失败,而不是 panic 或返回错误数据。
        let garbage = b"this is not a real DPAPI ciphertext blob, just bytes";
        let result = unprotect(garbage);
        assert!(matches!(result, Err(CryptoError::UnprotectFailed(_))));
    }
}
