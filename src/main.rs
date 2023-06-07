//!
//! Application entrypoint.
//!

mod util;

use crate::util::{execute_command, get_current_timestamp, is_file, is_linux, is_windows};

/// Query the latest tag of this repository.
fn execute_gh_release_list() -> Result<String, Box<dyn std::error::Error>> {
	green!("> gh release list");

	if is_windows() {
		let mut command = std::process::Command::new("gh.exe");
		let result = command.args(&["release", "list"]).stderr(std::process::Stdio::inherit()).output()?;
		if !result.status.success() {
			let code = result.status.code().unwrap();
			error!("process exited with code {}.", code);
			error!("Failed to retrieve the latest tag of the repository in github.com.");
			return Err("Command exited with error.".into());
		}
		let stdout = String::from_utf8(result.stdout)?;
		return Ok(stdout);
	} else if is_linux() {
		let mut command = std::process::Command::new("gh");
		let result = command.args(&["release", "list"]).stderr(std::process::Stdio::inherit()).output()?;
		if !result.status.success() {
			let code = result.status.code().unwrap();
			error!("process exited with code {}.", code);
			error!("Failed to retrieve the latest tag of the repository in github.com.");
			return Err("Command exited with error.".into());
		}
		let stdout = String::from_utf8(result.stdout)?;
		return Ok(stdout);
	} else {
		return Err("Unsupported OS.".into());
	}
}

/// Retrieve latest tag from gh command.
fn get_gh_current_tag() -> Result<String, Box<dyn std::error::Error>> {
	let stdout = execute_gh_release_list()?;

	let lines: Vec<&str> = stdout.split("\n").collect();

	for line in &lines {
		let line = line.trim();

		green!("> {}", line);

		if !line.contains("Latest") {
			info!("ignored. (no latest)");
			continue;
		}

		let items: Vec<&str> = line.split("\t").collect();
		if items.len() < 3 {
			info!("ignored. (invalid number of fields {})", items.len());
			continue;
		}

		// It is the "Latest" line.
		let tag = items[2];
		info!("FOUND latest release tagged as [{}].", tag);
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

fn straighten_command_string(params: &[&str]) -> String {
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

fn getenv(name: &str) -> String {
	let result = std::env::var(name);
	return result.unwrap_or_default();
}

/// Try to get the tag name from the environment variable GITHUB_REF.
fn try_get_tag_name() -> Result<Option<String>, Box<dyn std::error::Error>> {
	// May be Branch description or tag description.
	let tag = getenv("GITHUB_REF");

	let result = matches(&tag, r"^refs/tags/(.+)$")?;
	if result.len() != 1 {
		return Ok(None);
	}

	return Ok(Some(result[0].clone()));
}

fn generate_new_tag(new_tag: &str) -> Result<String, Box<dyn std::error::Error>> {
	if new_tag != "" {
		info!("NEXT TAG: [{}]", new_tag);

		return Ok(new_tag.to_string());
	} else if let Some(tag_name) = try_get_tag_name()? {
		// GITHUB_REF_NAME exists. Triggered by tagging on GitHub Actions.
		if tag_name == "" {
			error!("GITHUB_REF_NAME is empty.");
			return Err("Command exited with error.".into());
		}

		info!("NEXT TAG: [{}]", &tag_name);

		return Ok(tag_name);
	} else {
		// latest tag in releases.
		let latest_tag = get_gh_current_tag()?;

		// increment
		let mut next_tag = generate_tag(&latest_tag)?;
		if next_tag == "" {
			next_tag = "1".to_string();
		}

		info!("NEXT TAG: [{}]", &next_tag);

		return Ok(next_tag);
	}
}

/// Launch gh command to create release.
fn gh_release_create(dry_run: bool, new_tag: &str, title: &str, target: &str, notes: &str, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
	info!("files: {:?}", files);

	let mut params: Vec<&str> = vec!["gh", "release", "create"];

	let next_tag = generate_new_tag(new_tag)?;
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
		// Generate release notes automatically.
		params.push("--generate-notes");
	} else if is_file(notes) {
		// Argument is file path.
		params.push("--notes-file");
		params.push(notes);
	} else {
		// Argument is string.
		params.push("--notes");
		params.push(notes);
	}

	// ATTACHMENTS
	for file in &files {
		params.push(&file);
	}

	if dry_run {
		// Dry run.
		info!("CREATING RELEASE... (DRY-RUN)");

		let command_string = straighten_command_string(&params);
		green!("> {}", &command_string);
	} else {
		info!("CREATING RELEASE...");

		execute_command(&params)?;
	}

	return Ok(());
}

trait MatchHelper {
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

fn build_myself() -> Result<(), Box<dyn std::error::Error>> {
	info!("BUILDING...");

	// win/linux
	execute_command(&["cargo", "build", "--quiet", "--release"])?;

	return Ok(());
}

/// create release to publish.
fn make_publish(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
	build_myself()?;

	let crate_version = env!("CARGO_PKG_VERSION");

	if dry_run {
		info!("PUBLISHING... (DRY-RUN)");

		if is_windows() {
			println!(
				"cargo.exe run --quiet --release -- --title {} --file target\\release\\r-gh-create-release.exe",
				&crate_version
			);
		} else {
			println!(
				"cargo run --quiet --release -- --title {} --file target/release/r-gh-create-release",
				&crate_version
			);
		}
	} else {
		info!("PUBLISHING...");

		if is_windows() {
			execute_command(&[
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
		} else {
			execute_command(&[
				"cargo",
				"run",
				"--quiet",
				"--release",
				"--",
				"--title",
				&crate_version,
				"--file",
				"target/release/r-gh-create-release",
			])?;
		}
	}

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

	// Parse arguments.
	let mut options = getopts::Options::new();
	options.optflag("h", "help", "usage");
	options.optflag("", "publish", "go publish");
	options.optflag("", "dry-run", "dry run");
	options.opt("", "notes", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "tag", "create release using tag.", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "title", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "target", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "file", "string", "ARRAY", getopts::HasArg::Yes, getopts::Occur::Multi);

	let result = options.parse(args);
	if result.is_err() {
		eprint!("{}", options.usage(""));
		std::process::exit(1);
	}
	let input = result.unwrap();

	// Option: Dry run.
	let dry_run = input.opt_present("dry-run");

	if input.opt_present("help") {
		// ========== OPTIONAL: SHOW HELP ==========
		eprintln!("{}", options.usage(""));
	} else if input.opt_present("publish") {
		// ========== OPTIONAL: MAKE PUBLISH SELF ==========
		// Build once in release, and make self publish.
		let result = make_publish(dry_run);
		if result.is_err() {
			let reason = result.err().unwrap();
			error!("{}", reason);
			std::process::exit(1);
		}
	} else {
		// ========== DEFAULT: CREATE RELEASE ==========
		// Option: Use tag.
		let tag_name = input.get_string("tag");

		// option: Release title.
		let title = input.get_string("title");

		// option: Branch name.
		let target = input.get_string("target");

		// option: Release notes.
		//   --generate-notes will be used if this is empty.
		let notes = input.get_string("notes");

		// option: Attachments.
		let files: Vec<String> = input.get_strings("file");

		// Create release.
		let result = gh_release_create(dry_run, &tag_name, &title, &target, &notes, files);
		if result.is_err() {
			let reason = result.err().unwrap();
			error!("{}", reason);
			std::process::exit(1);
		}
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
