#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use tray_item::{IconSource, TrayItem};
#[cfg(target_os = "linux")]
use openscq30_lib::{
    device::OpenSCQ30Device,
    settings::{Setting, SettingId},
};
#[cfg(target_os = "linux")]
use crate::app::DebugOpenSCQ30Device;

pub struct TrayManager {
    #[cfg(target_os = "linux")]
    device: Arc<Mutex<Option<Arc<dyn OpenSCQ30Device + Send + Sync>>>>,
}

impl TrayManager {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            let device = Arc::new(Mutex::new(None));
            let device_clone = device.clone();
            
            thread::spawn(move || {
                // Try to create tray icon
                // On some systems, this might fail if system tray is not available
                // Try to use the application icon, fallback to empty if not available
                let icon = IconSource::Resource("com.oppzippy.OpenSCQ30");
                let _tray = match TrayItem::new("OpenSCQ30", icon) {
                    Ok(mut tray) => {
                        // Add menu items
                        tray.add_label("OpenSCQ30").ok();
                        tray.add_label("").ok(); // Separator
                        
                        // Battery info (updated periodically)
                        let _battery_label = tray.add_label("Battery: --").ok();
                        
                        tray.add_label("").ok(); // Separator
                        
                        // Sound mode section
                        tray.add_label("Sound Mode").ok();
                        let _noise_cancel = tray.add_menu_item("Toggle Noise Canceling", {
                            let device = device_clone.clone();
                            move || {
                                Self::toggle_setting(&device, SettingId::NoiseCancelingMode);
                            }
                        }).ok();
                        
                        let _ambient_sound = tray.add_menu_item("Cycle Ambient Sound", {
                            let device = device_clone.clone();
                            move || {
                                Self::cycle_ambient_sound(&device);
                            }
                        }).ok();
                        
                        tray.add_label("").ok(); // Separator
                        
                        // Equalizer section
                        tray.add_label("Equalizer Presets").ok();
                        let _eq_normal = tray.add_menu_item("Normal", {
                            let device = device_clone.clone();
                            move || {
                                Self::set_equalizer_preset(&device, "Normal");
                            }
                        }).ok();
                        
                        let _eq_bass = tray.add_menu_item("Bass", {
                            let device = device_clone.clone();
                            move || {
                                Self::set_equalizer_preset(&device, "Bass");
                            }
                        }).ok();
                        
                        let _eq_treble = tray.add_menu_item("Treble", {
                            let device = device_clone.clone();
                            move || {
                                Self::set_equalizer_preset(&device, "Treble");
                            }
                        }).ok();
                        
                        tray.add_label("").ok(); // Separator
                        
                        tray.add_menu_item("Quit", || {
                            std::process::exit(0);
                        }).ok();
                        
                        // Keep tray alive
                        loop {
                            thread::sleep(std::time::Duration::from_secs(60));
                        }
                    }
                    Err(e) => {
                        tracing::debug!("System tray not available: {e:?}");
                        return;
                    }
                };
            });
            
            Self { device }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Self {}
        }
    }
    
    pub fn update_device(&mut self, device: Option<&DebugOpenSCQ30Device>) {
        #[cfg(target_os = "linux")]
        {
            *self.device.lock().unwrap() = device.map(|d| d.0.clone());
        }
    }
    
    #[cfg(target_os = "linux")]
    fn toggle_setting(
        device: &Arc<Mutex<Option<Arc<dyn OpenSCQ30Device + Send + Sync>>>>,
        setting_id: SettingId,
    ) {
        let device_guard = device.lock().unwrap();
        if let Some(device_ref) = device_guard.as_ref() {
            if let Some(setting) = device_ref.setting(&setting_id) {
                if let Setting::Toggle { value } = setting {
                    let new_value = !value;
                    let device_clone = device_ref.clone();
                    drop(device_guard);
                    tokio::spawn(async move {
                        if let Err(e) = device_clone
                            .set_setting_values(vec![(setting_id, new_value.into())])
                            .await
                        {
                            tracing::error!("Failed to set setting: {e:?}");
                        }
                    });
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    fn cycle_ambient_sound(
        device: &Arc<Mutex<Option<Arc<dyn OpenSCQ30Device + Send + Sync>>>>,
    ) {
        let device_guard = device.lock().unwrap();
        if let Some(device_ref) = device_guard.as_ref() {
            if let Some(setting) = device_ref.setting(&SettingId::AmbientSoundMode) {
                if let Setting::Select { setting: select_setting, value: current_value } = setting {
                    // Find next option - options are directly Cow<str>, value is also Cow<str>
                    let current_idx = select_setting.options.iter()
                        .position(|opt| *opt == *current_value)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % select_setting.options.len();
                    let next_value = select_setting.options[next_idx].clone();
                    
                    let device_clone = device_ref.clone();
                    drop(device_guard);
                    tokio::spawn(async move {
                        if let Err(e) = device_clone
                            .set_setting_values(vec![(SettingId::AmbientSoundMode, next_value.into())])
                            .await
                        {
                            tracing::error!("Failed to set ambient sound mode: {e:?}");
                        }
                    });
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    fn set_equalizer_preset(
        device: &Arc<Mutex<Option<Arc<dyn OpenSCQ30Device + Send + Sync>>>>,
        preset_name: &str,
    ) {
        let device_guard = device.lock().unwrap();
        if let Some(device_ref) = device_guard.as_ref() {
            if let Some(setting) = device_ref.setting(&SettingId::PresetEqualizerProfile) {
                if let Setting::Select { setting: select_setting, .. } = setting {
                    // Find preset by name (case insensitive) - match against localized_options
                    if let Some((idx, _)) = select_setting.localized_options.iter()
                        .enumerate()
                        .find(|(_, label)| label.to_lowercase() == preset_name.to_lowercase())
                    {
                        let device_clone = device_ref.clone();
                        let value_clone = select_setting.options[idx].clone();
                        drop(device_guard);
                        tokio::spawn(async move {
                            if let Err(e) = device_clone
                                .set_setting_values(vec![(SettingId::PresetEqualizerProfile, value_clone.into())])
                                .await
                            {
                                tracing::error!("Failed to set equalizer preset: {e:?}");
                            }
                        });
                    }
                }
            }
        }
    }
    
    pub fn shutdown(self) {
        // Tray will be cleaned up when dropped
    }
}

