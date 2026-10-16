//! Compiled simulated provider for lifecycle/security tests. Never shipped.
use std::{io::{Read,Write,BufRead},time::Duration};
fn main() {
 let args:Vec<String>=std::env::args().skip(1).collect();
 if args.first().map(String::as_str)==Some("--forma-watch-agent") {
  #[cfg(unix)] {
   let mut lines=std::io::stdin().lock().lines();
   if let Some(Ok(pid))=lines.next(){if let Ok(pid)=pid.parse::<i32>() {let _=std::io::stdout().write_all(b"R"); let _=std::io::stdout().flush();if pid>1&&lines.next().is_none_or(|x|!matches!(x.as_deref(),Ok("done"))){unsafe extern "C"{fn kill(pid:i32,sig:i32)->i32;} unsafe{kill(-pid,9);}}}}
  }
  return;
 }
 if std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy().contains("incompatible") {
  if args.contains(&"--version".into()) {println!("codex-cli 0.1.0");} else {println!("Legacy CLI without required security flags");}
  return;
 }
 if args.contains(&"--version".into()){if std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy().contains("claude") {println!("2.1.0 (Claude Code)");} else {println!("codex-cli simulated-m05");}return;}
 if args.contains(&"--help".into()){for n in 0..260 {println!("Capability detail line {n}");}println!("--no-daemon --ask-for-approval --color --sandbox --json --ignore-user-config --ignore-rules --ephemeral --restricted --print --output-format --permission-mode --tools --allowedTools --strict-mcp-config --mcp-config --settings --verbose --no-session-persistence dontAsk stream-json");return;}
 if args==["login","status"] {println!("Logged in using ChatGPT");return;}
 if args==["auth","status"] {println!("{{\"loggedIn\":false}}");std::process::exit(1);}
 if args==["login"] || args==["auth","login"] {std::thread::sleep(Duration::from_secs(30));return;}
 if args.contains(&"--descendant".into()){std::thread::sleep(Duration::from_secs(3));std::fs::write("escaped-descendant.txt","bad").unwrap();return;}
 if args.first().map(String::as_str)==Some("--forma-test") {
  std::fs::write("independent-arguments.txt",args.join("\n")).unwrap();
  println!("independent test output");
  if args.contains(&"slow".into()) {std::thread::sleep(Duration::from_secs(30));}
  if args.contains(&"fail".into()) {std::process::exit(9);}
  return;
 }
 std::fs::write("process-id.txt",std::process::id().to_string()).unwrap();
 let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
 assert!(args.contains(&"never".into()) || args.contains(&"dontAsk".into()));assert!(!args.iter().any(|a|a.contains("bypass")));
 std::fs::write("received-args.txt",args.join("\n")).unwrap();
 println!("{{\"type\":\"task.started\"}}");println!("token: sk-abcdefghijk123456");
 println!("password=topsecret");
 println!("-----BEGIN RSA PRIVATE KEY-----\nSENSITIVE_PRIVATE_BODY\n-----END RSA PRIVATE KEY-----");
 if prompt.contains("SPAWN_DESCENDANT") {std::process::Command::new(std::env::current_exe().unwrap()).arg("--descendant").spawn().unwrap();}
 if prompt.contains("FLOOD_OUTPUT"){for _ in 0..20000 {println!("{}", "x".repeat(256));}}
 if prompt.contains("LONG_LINE"){println!("{}sk-abcdefghijk123456","x".repeat(20000));}
 if prompt.contains("CPU_EXECUTION") {let start=std::time::Instant::now();let mut n=1u64;while start.elapsed()<Duration::from_secs(2){for _ in 0..10000 {n=n.wrapping_mul(6364136223846793005).wrapping_add(1);}std::hint::black_box(n);}}
 if prompt.contains("SLOW_EXECUTION"){std::io::stdout().flush().unwrap();std::thread::sleep(Duration::from_secs(30));}
 if prompt.contains("FAIL_EXECUTION"){std::process::exit(7);}
 std::fs::write("agent-result.txt","simulated provider change\n").unwrap();
 println!("{{\"type\":\"task.completed\",\"tests\":\"not run\"}}");
}
