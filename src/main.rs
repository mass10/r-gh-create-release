pub fn get_current_timestamp() -> String {
	let date = chrono::Local::now();
	return format!("{}", date.format("%Y-%m-%d %H:%M:%S%.3f"));
}

/// システムのシェルを利用してコマンドを実行します。
fn execute_command(command: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
	let string = command.join(" ");
	println!("> {}", string);
	let result = std::process::Command::new("cmd").args(&["/C"]).args(command).output()?;
	if !result.status.success() {
		let code = result.status.code().unwrap();
		eprintln!("error: {}", code);
		return Err("".into());
	}
	eprint!("process exited with code: {}", result.status.code().unwrap());
	return Ok(());
}

/// Latest のタグを取得します。
fn get_gh_current_tag() -> Result<String, Box<dyn std::error::Error>> {
	let result = std::process::Command::new("cmd").args(&["/C"]).args(&["gh", "release", "list"]).output()?;
	if !result.status.success() {
		let code = result.status.code().unwrap();
		eprintln!("[error] error: {}", code);
		return Err("".into());
	}

	let stdout = String::from_utf8(result.stdout)?;
	let lines: Vec<&str> = stdout.split("\r\n").collect();
	if lines.len() < 2 {
		return Err("".into());
	}

	let line = lines[1];
	let items: Vec<&str> = line.split("\t").collect();
	if items.len() < 2 {
		return Err("".into());
	}

	let tag = items[0];
	return Ok(tag.to_string());
}

fn gh_release_create(title: &str, branch_name: &str, notes: &str, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
	// gh release create v0.3 --target main --generate-notes "bin\Release\LocalStoreExample1.exe"
	println!("[debug] files: {:?}", files);

	let mut params: Vec<&str> = vec!["gh", "release", "create"];

	// TAG
	println!("[debug] analyzing current tags...");
	let tag = get_gh_current_tag()?;
	println!("[debug] current tag: {}", tag);
	if !tag.starts_with("v") {
		eprint!("invalid tag: {}", tag);
		return Err("".into());
	}
	println!("[debug] current tag: {}", tag);
	let current_build_number: u32 = tag[1..].parse()?;
	let next_tag = format!("v{}", current_build_number + 1);
	params.push(&next_tag);

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
	}
	else {
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

fn main() {
	let mut options = getopts::Options::new();
	options.optflag("h", "help", "usage");
	options.opt("", "notes", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "title", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "branch", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "file", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);

	let result = options.parse(std::env::args().skip(1));
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
		eprintln!("{}", result.err().unwrap());
		return;
	}
}

// cargo run -- target\release\rcreate-release.exe
