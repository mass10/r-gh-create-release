//!
//! Utility functions.
//!

/// macro: print info text.
#[macro_export]
macro_rules! info {
	() => {
		println!("");
	};
	($($arg:tt)*) => {
		println!("[INFO] {}", format!($($arg)*));
	};
}

/// macro: print error text.
#[macro_export]
macro_rules! error {
	() => {
		println!("");
	};
	($($arg:tt)*) => {
		println!("[ERROR] {}", format!($($arg)*));
	};
}

/// Print text in green.
#[macro_export]
macro_rules! green {
	($($arg:tt)*) => {
		println!("{}", format!("\x1b[32m{}\x1b[0m", format!($($arg)*)));
	};
}

/// Get current timestamp as string.
pub fn get_current_timestamp() -> String {
	let date = chrono::Local::now();
	return format!("{}", date.format("%Y-%m-%d %H:%M:%S%.3f"));
}

/// Whether if file exists.
pub fn is_file(path: &str) -> bool {
	let path = std::path::Path::new(path);
	return path.exists();
}

/// Whether if the current OS is Windows.
pub fn is_windows() -> bool {
	return cfg!(target_os = "windows");
}

/// Whether if the current OS is Linux.
pub fn is_linux() -> bool {
	return cfg!(target_os = "linux");
}

/// Execute command in shell.
pub fn execute_command(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
	{
		let string = args.join(" ");
		green!("> {}", string);
	}

	if is_windows() {
		let (cmd_name, args) = args.split_first().unwrap();
		let mut command = std::process::Command::new(cmd_name);
		let result = command.args(args).spawn()?.wait()?;
		if !result.success() {
			let code = result.code().unwrap();
			error!("process exited with code {}.", code);
			error!("Failed to execute command.");
			return Err("Command exited with error.".into());
		}

		info!("process exited with code: {}", result.code().unwrap());
	} else if is_linux() {
		let (cmd_name, args) = args.split_first().unwrap();
		let mut command = std::process::Command::new(cmd_name);
		let result = command.args(args).spawn()?.wait()?;
		if !result.success() {
			let code = result.code().unwrap();
			error!("process exited with code {}.", code);
			error!("Failed to execute command.");
			return Err("Command exited with error.".into());
		}

		info!("process exited with code: {}", result.code().unwrap());
	} else {
		return Err("Unsupported OS.".into());
	}

	return Ok(());
}
