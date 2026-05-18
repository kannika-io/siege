use std::future::Future;
use std::pin::Pin;

use siege::{SeedError, SiegeError};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

pub trait Hook: Send + 'static {
    fn run(&mut self) -> impl Future<Output = Result<(), SiegeError>> + Send;
}

/// Object-safe version of [`Hook`] for use in trait objects.
pub(crate) trait DynHook: Send {
    fn run_boxed(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SiegeError>> + Send + '_>>;
}

impl<T: Hook> DynHook for T {
    fn run_boxed(&mut self) -> Pin<Box<dyn Future<Output = Result<(), SiegeError>> + Send + '_>> {
        Box::pin(self.run())
    }
}

impl Hook for Command {
    async fn run(&mut self) -> Result<(), SiegeError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let log_path = std::env::temp_dir().join(format!("siege-post-seed-hook-{timestamp}.log"));

        eprintln!("[post-seed-hook] running...");
        eprintln!("[post-seed-hook] log: {}", log_path.display());

        let mut child = self
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| SiegeError::Seed(SeedError::Failed(format!("post-seed hook failed to spawn: {e}"))))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let log_path_clone = log_path.clone();
        let output_handle = tokio::spawn(async move {
            let mut file = File::create(&log_path_clone).await.ok();

            if let Some(stdout) = stdout {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("[post-seed-hook] {line}");
                    if let Some(ref mut f) = file {
                        let _ = f.write_all(format!("{line}\n").as_bytes()).await;
                    }
                }
            }
        });

        let stderr_handle = tokio::spawn(async move {
            if let Some(stderr) = stderr {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("[post-seed-hook] {line}");
                }
            }
        });

        let status = child
            .wait()
            .await
            .map_err(|e| SiegeError::Seed(SeedError::Failed(format!("post-seed hook wait failed: {e}"))))?;

        let _ = output_handle.await;
        let _ = stderr_handle.await;

        if !status.success() {
            let code = status.code().unwrap_or(-1);
            eprintln!("[post-seed-hook] log: {}", log_path.display());
            return Err(SiegeError::Seed(SeedError::Failed(format!(
                "post-seed hook exited with code {code}"
            ))));
        }

        Ok(())
    }
}
