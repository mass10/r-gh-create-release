pub fn get_current_timestamp() -> String {
	let date = chrono::Local::now();
	return format!("{}", date.format("%Y-%m-%d %H:%M:%S%.3f"));
}

/// システムのシェルを利用してコマンドを実行します。
fn execute_command(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
	let string = args.join(" ");
	println!("> {}", string);

	let mut command = std::process::Command::new("cmd.exe");
	let result = command.args(&["/C"]).args(args).spawn()?.wait()?;
	if !result.success() {
		let code = result.code().unwrap();
		println!("[ERROR] process exited with code: {}", code);
		return Err("コマンドは正常終了しませんでした。".into());
	}

	println!("[DEBUG] process exited with code: {}", result.code().unwrap());
	return Ok(());
}

/// Latest のタグを取得します。
fn get_gh_current_tag() -> Result<String, Box<dyn std::error::Error>> {
	let mut command = std::process::Command::new("cmd.exe");
	let result = command.args(&["/C"]).args(&["gh", "release", "list"]).output()?;
	if !result.status.success() {
		let code = result.status.code().unwrap();
		println!("[ERROR] process exited with exit code: [{}]", code);
		return Err("コマンドは正常終了しませんでした。".into());
	}

	let stdout = String::from_utf8(result.stdout)?;

	let lines: Vec<&str> = stdout.split("\n").collect();

	for line in &lines {
		let line = line.trim();

		println!("> {}", line);

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
		println!("[DEBUG] Latest release tagged as [{}].", tag);
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

fn gh_release_create(title: &str, branch_name: &str, notes: &str, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
	// gh release create v0.3 --target main --generate-notes "bin\Release\LocalStoreExample1.exe"
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

	// RELEASE TITLE
	params.push("--title");
	let release_title = if title == "" {
		let value = format!("release {}", get_current_timestamp());
		value
	} else {
		title.to_string()
	};
	params.push(&release_title);

	// BRANCH (TODO: ※draft を考慮)
	if branch_name == "" {
		params.push("--target");
		params.push("main");
	} else if branch_name == "main" {
		params.push("--target");
		params.push("main");
	} else {
		params.push("--target");
		params.push(&branch_name);
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

trait StringParam {
	fn get_string(&self, name: &str) -> String;
}

impl StringParam for getopts::Matches {
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

trait StringUtility {
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

fn main() {
	let args: Vec<String> = std::env::args().skip(1).collect();

	// 第一引数
	let first_request = args.at(0);

	if first_request == "--publish" {
		// 自分自身をビルドしてリリース
		let result = make_publish();
		if result.is_err() {
			println!("[ERROR] {}", result.err().unwrap());
		}
		return;
	}

	let mut options = getopts::Options::new();
	options.optflag("h", "help", "usage");
	options.opt("", "notes", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "title", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "branch", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "file", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);

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

	let title = if input.opt_present("title") {
		input.opt_str("title").unwrap()
	} else {
		"".to_string()
	};

	let branch_name = if input.opt_present("branch") {
		input.opt_str("branch").unwrap()
	} else {
		"".to_string()
	};

	let notes = if input.opt_present("notes") {
		input.opt_str("notes").unwrap()
	} else {
		"".to_string()
	};

	let files: Vec<String> = if input.opt_present("file") { input.opt_strs("file") } else { vec![] };

	let result = gh_release_create(&title, &branch_name, &notes, files);
	if result.is_err() {
		println!("[ERROR] {}", result.err().unwrap());
		return;
	}
}
