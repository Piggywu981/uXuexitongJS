use crate::config::CONFIG;
use crate::core::{
    script::load_on,
    url::{classify, Type},
};

use anyhow::anyhow;
use parking_lot::Mutex;
use tauri::{LogicalPosition, LogicalSize, Manager, Url, Webview, WebviewBuilder, WebviewUrl};

#[derive(Default)]
pub struct UrlStack(Mutex<(Vec<Url>, usize)>);

fn chaoxing_geometry(
    window: &tauri::Window,
) -> Result<(LogicalPosition<f64>, LogicalSize<f64>), Box<dyn std::error::Error>> {
    let logical_size: LogicalSize<f64> =
        LogicalSize::from_physical(window.inner_size()?, window.scale_factor()?);

    Ok((
        LogicalPosition::new(logical_size.width * 0.51, logical_size.height * 0.46),
        LogicalSize::new(logical_size.width * 0.48, logical_size.height * 0.48),
    ))
}

/// 根据当前窗口内容区尺寸重新对齐超星原生 WebView。
pub fn sync_chaoxing(window: &tauri::Window) -> Result<(), Box<dyn std::error::Error>> {
    let Some(webview) = window.get_webview("chaoxing") else {
        return Ok(());
    };
    let (position, size) = chaoxing_geometry(window)?;

    webview.set_position(position)?;
    webview.set_size(size)?;
    log::debug!(
        "重新同步超星WebView几何 - 位置: ({}, {}), 大小: ({}x{})",
        position.x,
        position.y,
        size.width,
        size.height
    );
    Ok(())
}

impl UrlStack {
    pub fn push(&self, url: Url) {
        if url.as_str() == "about:blank" {
            return;
        }
        let (urls, index) = &mut *self.0.lock();
        if urls.get(*index) != Some(&url) {
            urls.truncate(*index + 1);
            urls.push(url);
            *index = urls.len().saturating_sub(1);
        }
    }

    pub fn can_back(&self) -> bool {
        let (_, index) = &*self.0.lock();
        *index > 0
    }

    pub fn can_forward(&self) -> bool {
        let (urls, index) = &*self.0.lock();
        *index + 1 < urls.len()
    }

    pub fn back(&self) -> Option<Url> {
        if !self.can_back() {
            return None;
        }
        let (urls, index) = &mut *self.0.lock();
        *index -= 1;
        urls.get(*index).cloned()
    }

    pub fn current(&self) -> Option<Url> {
        let (urls, index) = &*self.0.lock();
        urls.get(*index).cloned()
    }

    pub fn forward(&self) -> Option<Url> {
        if !self.can_forward() {
            return None;
        }
        let (urls, index) = &mut *self.0.lock();
        *index += 1;
        urls.get(*index).cloned()
    }
}

pub fn init_on(window: &tauri::Window, label: &str) -> Result<Webview, Box<dyn std::error::Error>> {
    log::debug!("开始初始化Webview [{}]", label);
    let logical_size: LogicalSize<f64> =
        tauri::LogicalSize::from_physical(window.inner_size()?, window.scale_factor()?);

    let (builder, position, size) = match label {
        "main" => (
            WebviewBuilder::new(label, WebviewUrl::App("main.html".into()))
                .background_color((0, 0, 0, 0).into())
                .devtools(cfg!(debug_assertions)),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(logical_size.width, logical_size.height),
        ),
        "mask" => (
            WebviewBuilder::new(label, WebviewUrl::App("mask.html".into()))
                .background_color((0, 0, 0, 0).into())
                .devtools(true),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(logical_size.width, logical_size.height),
        ),
        "chaoxing" => (
            WebviewBuilder::new(
                label,
                WebviewUrl::External(CONFIG.metadata.home_url.clone()),
            )
        }
        _ => return Err(anyhow!("未知的Webview标签").into()),
    };

    log::debug!(
        "Webview [{}] 初始化参数 - 位置: ({}, {}), 大小: ({}x{})",
        label,
        position.x,
        position.y,
        size.width,
        size.height
    );
    Ok(window.add_child(builder, position, size)?)
}
