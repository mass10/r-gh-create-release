//!
//! Application entrypoint.
//!

mod application;
mod util;

/// Report error.
fn report_error(error: Box<dyn std::error::Error>) {
	let reason = error.to_string();
	if reason != "" {
		error!("{}", reason);
	}
	info!("Command exited with error.");
}

/// Create commandline options.
fn create_commandline_options() -> getopts::Options {
	let mut options = getopts::Options::new();
	options.optflag("h", "help", "Show usage.");
	options.optflag("", "publish", "Create a new release of gh-create-release. (For maintenance)");
	options.optflag("", "dry-run", "dry run.");
	options.opt(
		"",
		"determine-version-from",
		"Determines version string from file. (Cargo.toml, etc...)",
		"STRING",
		getopts::HasArg::Yes,
		getopts::Occur::Optional,
	);
	options.opt("", "notes", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "tag", "create release using tag.", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "title", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "target", "string", "STRING", getopts::HasArg::Yes, getopts::Occur::Optional);
	options.opt("", "file", "string", "ARRAY", getopts::HasArg::Yes, getopts::Occur::Multi);

	return options;
}

/// Show usage.
fn usage(options: &getopts::Options) {
	let pkg_name = env!("CARGO_PKG_NAME");
	let head = format!("{}: create github release", pkg_name);
	eprint!("{}", options.usage(&head));
}

/// Entrypoint of Rust application.
fn main() {
	use util::MatchHelper;

	let args: Vec<String> = std::env::args().skip(1).collect();

	// Parse arguments.
	let options = create_commandline_options();
	let result = options.parse(args);
	if result.is_err() {
		let error = result.err().unwrap();
		eprintln!("{}", error);
		eprintln!();
		usage(&options);
		std::process::exit(1);
	}
	let input = result.unwrap();

	// option: Dry run.
	let dry_run = input.opt_present("dry-run");

	if input.opt_present("help") {
		// ========== OPTIONAL: SHOW HELP ==========
		usage(&options);
	} else if input.opt_present("publish") {
		// ========== OPTIONAL: MAKE PUBLISH SELF (FOR MAINTENANCE) ==========
		// Build once in release, and make self publish.
		let result = application::make_self_published(dry_run);
		if result.is_err() {
			report_error(result.err().unwrap());
			std::process::exit(1);
		}
	} else {
		// ========== DEFAULT: CREATE RELEASE ==========
		// option: Use tag.
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

		// option: Determine version from file.
		let determine_version_from = input.get_string("determine-version-from");

		// Create release.
		let result = application::gh_release_create(dry_run, &tag_name, &title, &target, &notes, &determine_version_from, &files);
		if result.is_err() {
			report_error(result.err().unwrap());
			std::process::exit(1);
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::application::generate_tag;

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
