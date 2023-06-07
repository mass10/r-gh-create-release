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

/// strtoul.
pub fn parse_uint(text: &str) -> u32 {
	let number: Result<u32, _> = text.parse();
	if number.is_err() {
		return 0;
	}
	return number.unwrap();
}

/// For trace.
pub fn straighten_command_string(params: &[&str]) -> String {
	let mut result = String::new();
	for param in params {
		if result.len() > 0 {
			result.push(' ');
		}
		if param.contains(" ") {
			result.push('"');
			result.push_str(param);
			result.push('"');
			continue;
		}
		result.push_str(param);
	}
	return result;
}

pub fn getenv(name: &str) -> String {
	let result = std::env::var(name);
	return result.unwrap_or_default();
}

/// Helpers for getopts::Matches.
pub trait MatchHelper {
	fn get_string(&self, name: &str) -> String;

	fn get_strings(&self, name: &str) -> Vec<String>;
}

impl MatchHelper for getopts::Matches {
	fn get_string(&self, name: &str) -> String {
		if !self.opt_present(name) {
			return "".to_string();
		}
		let status = self.opt_str(name);
		if status.is_none() {
			return "".to_string();
		}
		return status.unwrap();
	}

	fn get_strings(&self, name: &str) -> Vec<String> {
		if !self.opt_present(name) {
			return Vec::new();
		}
		return self.opt_strs(name);
	}
}

/// utilities for strings.
pub trait StringUtility {
	/// get string at index.
	fn at(&self, index: usize) -> &str;
}

impl StringUtility for Vec<String> {
	fn at(&self, index: usize) -> &str {
		if self.len() <= index {
			return "";
		}
		return &self[index];
	}
}

pub fn matches(string_value: &str, expression: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let expression = regex::Regex::new(&expression);
	if expression.is_err() {
		error!("regex compilation error. {}", expression.err().unwrap());
		return Err("Command exited with error.".into());
	}
	let expression = expression.unwrap();

	// try to capture by "(...)".
	let capture_result = expression.captures(&string_value);
	if capture_result.is_none() {
		info!("NOT MATCHED for expression [{}].", expression);
		return Ok(Vec::new());
	}

	info!("MATCHED for expression [{}].", expression);

	// capture result
	let capture_result = capture_result.unwrap();

	let mut result: Vec<String> = vec![];

	let mut index = 0;

	for e in capture_result.iter() {
		if index == 0 {
			// Skip the first element that is not a capture.
			index += 1;
			continue;
		}
		let matched = e.unwrap();
		let string = matched.as_str().to_string();
		result.push(string.to_string());
		index += 1;
	}

	return Ok(result);
}