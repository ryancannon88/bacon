pub static DEFAULT_PACKAGE_CONFIG: &str = r#"
# This is a configuration file for the bacon tool
# More info at https://github.com/Canop/bacon

default_job = "check"

[jobs]

[jobs.check]
command = ["cargo", "check", "--color", "always"]
need_stdout = false

[jobs.check-all]
command = ["cargo", "check", "--all-targets", "--color", "always"]
need_stdout = false
watch = ["tests", "benches", "examples"]

[jobs.clippy]
command = ["cargo", "clippy", "--color", "always"]
need_stdout = false

[jobs.test]
command = ["cargo", "test", "--color", "always"]
need_stdout = true
watch = ["tests"]

"#;
