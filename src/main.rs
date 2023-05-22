///
pub fn magenta<T: std::fmt::Display>(s: T) -> String {
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
	println!("{}", magenta(format!("> {}", string)));

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
	println!("{}", magenta("> gh release list"));

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

		println!("{}", magenta(format!("> {}", line)));

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

/// Launch gh command to create release.
fn gh_release_create(title: &str, target: &str, notes: &str, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
	println!("[DEBUG] files: {:?}", files);

	let mut params: Vec<&str> = vec!["gh", "release", "create"];

	// TAG
	let mut tag = get_gh_current_tag()?;
	if tag == "" {
		tag = "v0".to_string()
	} else if !tag.starts_with("v") {
		tag = "v0".to_string()
	}

	let current_build_number: u32 = parse_uint(&tag[1..]);
	let next_tag = format!("v{}", current_build_number + 1);
	params.push(&next_tag);

	println!("[DEBUG] creating next tag: [{}]", &next_tag);

	// title
	params.push("--title");
	let release_title = if title == "" {
		let value = format!("{}, release, {}", &next_tag, get_current_timestamp());
		value
	} else {
		title.to_string()
	};
	params.push(&release_title);

	// branch (TODO: ※Recognize draft release)
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

	execute_command(&[
		"cmd.exe",
		"/C",
		"cargo.exe",
		"run",
		"--quiet",
		"--",
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
		return;
	}
	let input = result.unwrap();

	if input.opt_present("help") {
		eprint!("{}", options.usage(""));
		return;
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
		return;
	}
}
