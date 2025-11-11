#![cfg(feature = "usb")]

//! 共通: 実機テスト用ヘルパー
//!
//! このファイルは `--features usb` でコンパイルされる実機テストに
//! 共通で使える関数を提供します。主な目的はテスト中に PaSoRi を安全に
//! open/initialize して、デバイスが無い環境（CI 等）では `Ok(None)` を返すことです。

use libpafe::transport::usb::UsbTransport;
use libpafe::{Error, Result, device, transport};

/// PaSoRi を開いて初期化した `Device<Initialized>` を返す。
///
/// - Ok(Some(device)) : デバイスが見つかり初期化に成功
/// - Ok(None) : デバイスが見つからない（CI 等では許容）
/// - Err(e) : その他の致命的なエラー
pub fn open_and_initialize_device() -> Result<Option<device::Device<device::Initialized>>> {
    match UsbTransport::open() {
        Ok(transport) => {
            let boxed: Box<dyn transport::traits::Transport> = Box::new(transport);
            let device = device::Device::new_with_transport(boxed)?;
            let initialized = device.initialize()?;
            Ok(Some(initialized))
        }
        Err(Error::DeviceNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}
