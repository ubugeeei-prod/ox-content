use crate::Result;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::Write;

pub struct Terminal {
    was_raw: bool,
    pub stopped: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[cfg(unix)]
    signals: Vec<signal_hook::SigId>,
}

impl Terminal {
    pub fn open() -> Result<Self> {
        let mut guard = Self {
            was_raw: terminal::is_raw_mode_enabled()?,
            stopped: std::sync::Arc::default(),
            #[cfg(unix)]
            signals: Vec::new(),
        };
        #[cfg(unix)]
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            guard.signals.push(signal_hook::flag::register(signal, guard.stopped.clone())?);
        }
        terminal::enable_raw_mode()?;
        execute!(std::io::stdout(), EnterAlternateScreen, Hide)?;
        std::io::stdout().flush()?;
        Ok(guard)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if !self.was_raw {
            let _ = terminal::disable_raw_mode();
        }
        let _ = execute!(std::io::stdout(), Show, LeaveAlternateScreen);
        let _ = std::io::stdout().flush();
        #[cfg(unix)]
        for signal in &self.signals {
            signal_hook::low_level::unregister(*signal);
        }
    }
}
