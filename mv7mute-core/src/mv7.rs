use hidapi::{HidApi, HidDevice};
use std::time::{Duration, Instant};

const VID: u16 = 0x14ED;
const PID: u16 = 0x1012;
const INTERFACE: i32 = 3;
const REPORT_SIZE: usize = 64;

pub trait Transport {
    fn send(&self, cmd: &str) -> Result<(), String>;
    fn read(&self, timeout_ms: i32) -> Result<Option<String>, String>;
}

pub struct HidTransport {
    device: HidDevice,
}

pub struct Mv7<T: Transport = HidTransport> {
    transport: T,
    was_locked: bool,
    restore_lock_armed: bool,
}

impl Transport for HidTransport {
    fn send(&self, cmd: &str) -> Result<(), String> {
        let mut report = [0u8; REPORT_SIZE + 1];
        let bytes = cmd.as_bytes();
        let len = bytes.len().min(REPORT_SIZE);
        report[1..=len].copy_from_slice(&bytes[..len]);
        self.device
            .write(&report)
            .map_err(|e| format!("HID write failed ({cmd:?}): {e}"))?;
        Ok(())
    }

    fn read(&self, timeout_ms: i32) -> Result<Option<String>, String> {
        let mut buf = [0u8; REPORT_SIZE];
        let len = self
            .device
            .read_timeout(&mut buf, timeout_ms)
            .map_err(|e| format!("HID read failed: {e}"))?;
        if len == 0 {
            return Ok(None);
        }

        let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
        let message = String::from_utf8_lossy(&buf[..end]).trim().to_string();
        if message.is_empty() {
            Ok(None)
        } else {
            Ok(Some(message))
        }
    }
}

impl Mv7<HidTransport> {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("HID init failed: {e}"))?;

        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VID && d.product_id() == PID && d.interface_number() == INTERFACE
            })
            .ok_or_else(|| {
                format!("MV7 not found (VID={VID:#06x} PID={PID:#06x} interface={INTERFACE})")
            })?;

        let device = info
            .open_device(&api)
            .map_err(|e| format!("Failed to open MV7: {e}"))?;

        Ok(Self::new(HidTransport { device }))
    }
}

impl<T: Transport> Mv7<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            was_locked: false,
            restore_lock_armed: false,
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        self.send("su adm")?;
        self.wait_for("su=adm", Duration::from_secs(3))?;
        self.send("bootDSP C")?;
        self.wait_for("dspBooted", Duration::from_secs(5))?;
        self.was_locked = self.get_lock()?;
        self.restore_lock_armed = self.was_locked;
        if self.was_locked {
            self.set_lock(false)?;
        }
        Ok(())
    }

    pub fn restore_lock(&mut self) -> Result<(), String> {
        if self.restore_lock_armed {
            self.set_lock(true)?;
            self.restore_lock_armed = false;
        }
        Ok(())
    }

    pub fn cancel_restore_lock(&mut self) {
        self.restore_lock_armed = false;
    }

    pub fn was_locked(&self) -> bool {
        self.was_locked
    }

    pub fn get_lock(&self) -> Result<bool, String> {
        self.send("lock")?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Some(msg) = self.read(200)? {
                if msg.contains("lock=on") {
                    return Ok(true);
                }
                if msg.contains("lock=off") {
                    return Ok(false);
                }
            }
        }
        Err("Timeout waiting for lock response".to_string())
    }

    pub fn set_lock(&self, locked: bool) -> Result<(), String> {
        let cmd = if locked { "lock on" } else { "lock off" };
        self.send(cmd)
    }

    pub fn get_mute(&self) -> Result<bool, String> {
        self.send("micMute")?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Some(msg) = self.read(200)? {
                if msg.contains("micMute=on") {
                    return Ok(true);
                }
                if msg.contains("micMute=off") {
                    return Ok(false);
                }
            }
        }
        Err("Timeout waiting for micMute response".to_string())
    }

    pub fn set_mute(&self, muted: bool) -> Result<(), String> {
        let cmd = if muted { "micMute on" } else { "micMute off" };
        self.send(cmd)
    }

    pub fn toggle(&self) -> Result<bool, String> {
        let current = self.get_mute()?;
        let new_state = !current;
        self.set_mute(new_state)?;
        Ok(new_state)
    }

    fn send(&self, cmd: &str) -> Result<(), String> {
        self.transport.send(cmd)
    }

    fn read(&self, timeout_ms: i32) -> Result<Option<String>, String> {
        self.transport.read(timeout_ms)
    }

    fn wait_for(&self, needle: &str, timeout: Duration) -> Result<(), String> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(msg) = self.read(200)?
                && msg.contains(needle)
            {
                return Ok(());
            }
        }
        Err(format!("Timeout waiting for {needle:?}"))
    }
}

impl<T: Transport> Drop for Mv7<T> {
    fn drop(&mut self) {
        let _ = self.restore_lock();
    }
}
