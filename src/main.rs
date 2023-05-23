/// Print text in green.
pub fn green<T: std::fmt::Display>(s: T) -> String {
	return format!("\x1b[32m{}\x1b[0m", s);
}

/// Get current timestamp as string.
pub fn get_current_timestamp() -> String {
	let date = chrono::Local::now();
	return format!("{}", date.format("%Y-%m-%d %H:%M:%S%.3f"));
}

/// Execute command in shell.
fn execute_command(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
	let string = args.join(" ");
	println!("{}", green(format!("> {}", string)));

	let mut command = std::process::Command::new("cmd.exe");
	let result = command.args(&["/C"]).args(args).spawn()?.wait()?;
	if !result.success() {
		let code = result.code().unwrap();
		println!("[ERROR] process exited with code {}.", code);
		return Err("Failed to launch command.".into());
	}

	println!("[DEBUG] process exited with code: {}", result.code().unwrap());
	return Ok(());
}

/// Retrieve latest tag from gh command.
fn get_gh_current_tag() -> Result<String, Box<dyn std::error::Error>> {
	println!("{}", green("> gh release list"));

	let mut command = std::process::Command::new("cmd.exe");
	let result = command.args(&["/C"]).args(&["gh", "release", "list"]).output()?;
	if !result.status.success() {
		let code = result.status.code().unwrap();
		println!("[ERROR] process exited with code {}.", code);
		return Err("Failed to retrieve the latest tag of the repository in github.com.".into());
	}

	let stdout = String::from_utf8(result.stdout)?;

	let lines: Vec<&str> = stdout.split("\n").collect();

	for line in &lines {
		let line = line.trim();

		println!("{}", green(format!("> {}", line)));

		if !line.contains("Latest") {
			println!("[DEBUG] ignored. (no latest)");
			continue;
		}

		let items: Vec<&str> = line.split("\t").collect();
		if items.len() < 3 {
			println!("[DEBUG] ignored. (invalid number of fields {})", items.len());
			continue;
		}

		// FOUND latest line.
		let tag = items[2];
		println!("[DEBUG] Found latest release tagged as [{}].", tag);
		return Ok(tag.to_string());
	}

	// NO valid lines.
	return Ok("".to_string());
}

fn parse_uint(text: &str) -> u32 {
	let number: Result<u32, _> = text.parse();
	if number.is_err() {
		return 0;
	}
	let number = number.unwrap();
	return number;
}

fn matches(string_value: &str, expression: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let expression = regex::Regex::new(&expression);
	if expression.is_err() {
		eprint!("ERROR: regex compile error. {}", expression.err().unwrap());
		return Err("".into());
	}
	let expression = expression.unwrap();

	// try to capture by "(...)".
	let capture_result = expression.captures(&string_value);
	if capture_result.is_none() {
		eprintln!("not match for exprtession [{}].", expression);
		return Ok(Vec::new());
	}

	// capture result
	let capture_result = capture_result.unwrap();

	let mut result: Vec<String> = vec![];

	let mut index = 0;

	for e in capture_result.iter() {
		if index == 0 {
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

/// Generate a new tag from the current tag.
fn generate_tag(tag: &str) -> Result<String, Box<dyn std::error::Error>> {
	// version: v#.#.#
	let part = matches(&tag, r"^v(\d+)\.(\d+)\.(\d+)$")?;
	if part.len() == 3 {
		let major: u32 = parse_uint(&part[0]);
		let minor: u32 = parse_uint(&part[1]);
		let patch: u32 = parse_uint(&part[2]);
		let next_tag = format!("v{}.{}.{}", major, minor, patch + 1);
		return Ok(next_tag);
	};

	// version: #.#.#
	let part = matches(&tag, r"^(\d+)\.(\d+)\.(\d+)$")?;
	if part.len() == 3 {
		let major: u32 = parse_uint(&part[0]);
		let minor: u32 = parse_uint(&part[1]);
		let patch: u32 = parse_uint(&part[2]);
		let next_tag = format!("{}.{}.{}", major, minor, patch + 1);
		return Ok(next_tag);
	};

	// version: v#
	let part = matches(&tag, r"^v(\d+)$")?;
	if part.len() == 1 {
		let major: u32 = parse_uint(&part[0]);
		let next_tag = format!("v{}", major + 1);
		return Ok(next_tag);
	};

	// version: #
	let part = matches(&tag, r"^(\d+)$")?;
	if part.len() == 1 {
		let major: u32 = parse_uint(&part[0]);
		let next_tag = format!("{}", major + 1);
		return Ok(next_tag);
	};

	return Ok("".to_string());
}

/// Launch gh command to create release.
fn gh_release_create(title: &str, target: &str, notes: &str, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
	println!("[DEBUG] files: {:?}", files);

	let mut params: Vec<&str> = vec!["gh", "release", "create"];

	// LATEST TAG
	let latest_tag = get_gh_current_tag()?;

	// increment
	let mut next_tag = generate_tag(&latest_tag)?;
	if next_tag == "" {
		next_tag = "1".to_string();
	}
	println!("[DEBUG] creating next tag: [{}]", &next_tag);

	params.push(&next_tag);

	// TITLE
	params.push("--title");
	let release_title = if title == "" {
		let value = format!("{}, release, {}", &next_tag, get_current_timestamp());
		value
	} else {
		title.to_string()
	};
	params.push(&release_title);

	// BRANCH (TODO: ※Recognize draft release)
	if target == "" {
		params.push("--target");
		params.push("main");
	} else if target == "main" {
		params.push("--target");
		params.push("main");
	} else {
		params.push("--target");
		params.push(&target);
	}

	// RELEASE NOTES
	if notes == "" {
		params.push("--generate-notes");
	} else {
		params.push("--notes");
		params.push(notes);
	}

	// ATTACHMENTS
	for file in &files {
		params.push(&file);
	}

	println!("[DEBUG] calling gh command.");

	execute_command(&params)?;

	return Ok(());
}

trait MatchHelper {
	fn get_string(&self, name: &str) -> String;
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
}

/// create release to publish.
fn make_publish() -> Result<(), Box<dyn std::error::Error>> {
	println!("[INFO] BUILDING...");

	execute_command(&["cmd.exe", "/C", "cargo.exe", "build", "--quiet", "--release"])?;

	println!("[INFO] PUBLISHING...");

	let crate_version = env!("CARGO_PKG_VERSION");

	execute_command(&[
		"cmd.exe",
		"/C",
		"cargo.exe",
		"run",
		"--quiet",
		"--release",
		"--",
		"--title",
		&crate_version,
		"--file",
		"target\\release\\r-gh-create-release.exe",
	])?;

	return Ok(());
}

/// utilities for strings.
trait StringUtility {
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

/// Entrypoint of Rust application.
fn main() {
	let args: Vec<String> = std::env::args().skip(1).collect();

	// Retrieve the first argument.
	let first_request = args.at(0);

	if first_request == "--publish" {
		// Build self, and make publish.
		let result = make_publish();
		if result.is_err() {
			println!("[ERROR] {}", result.err().unwrap());
			std::process::exit(1);
		}
		return;
	}

	// Parse arguments.
	let mut options = getopts::Options::new();
	options.optflag("h", "help", "usage");
	options.opt("", "notes", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "title", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "target", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "file", "string", "ARRAY", getopts::HasArg::Yes, getopts::Occur::Multi);

	let result = options.parse(args);
	if result.is_err() {
		eprint!("{}", options.usage(""));
		std::process::exit(1);
	}
	let input = result.unwrap();

	if input.opt_present("help") {
		eprint!("{}", options.usage(""));
		std::process::exit(1);
	}

	// Get arguments.
	let title = input.get_string("title");
	let target = input.get_string("target");
	let notes = input.get_string("notes");
	let files: Vec<String> = if input.opt_present("file") { input.opt_strs("file") } else { vec![] };

	// Create release.
	let result = gh_release_create(&title, &target, &notes, files);
	if result.is_err() {
		println!("[ERROR] {}", result.err().unwrap());
		std::process::exit(1);
	}
}

#[cfg(test)]
mod tests {
	use crate::generate_tag;

	#[test]
	fn test_generating_new_tag() {
		assert_eq!(generate_tag("").unwrap(), "".to_owned());

		assert_eq!(generate_tag("1").unwrap(), "2".to_owned());
		assert_eq!(generate_tag("2").unwrap(), "3".to_owned());
		assert_eq!(generate_tag("10").unwrap(), "11".to_owned());
		assert_eq!(generate_tag("99").unwrap(), "100".to_owned());

		assert_eq!(generate_tag("v0").unwrap(), "v1".to_owned());
		assert_eq!(generate_tag("v1002").unwrap(), "v1003".to_owned());

		assert_eq!(generate_tag("0.0.0").unwrap(), "0.0.1".to_owned());
		assert_eq!(generate_tag("0.0.1").unwrap(), "0.0.2".to_owned());

		assert_eq!(generate_tag("v1.0.567").unwrap(), "v1.0.568".to_owned());
	}
}
