use base64::Engine;
use std::env;

trait GenConfigString {
	fn gen_clash_config(&self) -> String;
}

#[derive(Debug)]
struct ShadowsocksEntry {
	entry_type: String,
	name: String,
	server: String,
	port: u16,
	cipher: String,
	password: String,
	plugin: String,
	udp: bool
}

impl GenConfigString for ShadowsocksEntry {
	fn gen_clash_config(&self) -> String {
		let mut config = String::new();
		config.push_str(&format!("  - {{name: {}, ", self.name));
		config.push_str(&format!("type: {}, ", self.entry_type));
		config.push_str(&format!("server: {}, ", self.server));
		config.push_str(&format!("port: {}, ", self.port));
		config.push_str(&format!("cipher: {}, ", self.cipher));
		config.push_str(&format!("password: {}, ", self.password));
		config.push_str(&format!("udp: {}, ", self.udp));
		
		if self.plugin.len() > 0 {
			config.push_str("plugin-opts: {");
			let plugin_opts = self.plugin.split(';').map(|kv| {
				let mut kv_parts = kv.splitn(2, '=');
				let k = match kv_parts.next() {
					Some(k) => k,
					None => {
						println!("[-] Failed to get key from plugin opts: {}", kv);
						return "".to_string();
					}
				};
				let v = match kv_parts.next() {
					Some(v) => v,
					None => {
						println!("[-] Failed to get value from plugin opts: {}", kv);
						return "".to_string();
					}
				};
				format!("{}: {}", k, v)
			}).collect::<Vec<String>>().join(", ");
			config.push_str(&plugin_opts);
			config.push_str("}, ");
		}
		// config.push_str(&format!("plugin: {}}}\n", self.plugin));

		config
	}
}

#[derive(Debug)]
enum ProxyNodeProtocol {
	Shadowsocks(ShadowsocksEntry),
}

#[derive(Debug)]
struct ProxyNodeEntry {
	// uri: String,
	protocol: ProxyNodeProtocol
}

impl ProxyNodeEntry {
	fn parse(uri: &str) -> Option<ProxyNodeEntry> {
		let decoded_uri = match urlencoding::decode(uri) {
			Ok(u) => u,
			Err(e) => {
				println!("[-] Failed to decode uri: {}, error: {}", uri, e);
				return None;
			}
		};
		
		let url = match url::Url::parse(&decoded_uri) {
			Ok(u) => u,
			Err(e) => {
				println!("[-] Failed to parse uri: {}, error: {}", uri, e);
				return None;
			}
		};

		match url.scheme() {
			"ss" => {
				let ss_entry = match ProxyNodeEntry::to_ss_entry(url) {
					Some(entry) => entry,
					None => {
						println!("[-] Failed to parse ss entry: {}", uri);
						return None;
					}
				};
				Some(ProxyNodeEntry {
					// uri: uri.to_string(),
					protocol: ProxyNodeProtocol::Shadowsocks(ss_entry)
				})
			},
			_ => {
				println!("[-] Unsupported protocol: {}", decoded_uri);
				None
			}
		}
	}

	// ss://YWVzLTEyOC1nY206aGFNTE1YaXJCeW42ckdWaA@app-zhihu.zhihu.win:20031/?plugin=simple-obfs%3Bobfs%3Dhttp%3Bobfs-host%3D5739b1aa758b.microsoft.com#%F0%9F%87%A6%F0%9F%87%B6%20%E5%8D%97%E6%9E%81%E4%B8%A83x%20AQ
	fn to_ss_entry(parsed_url: url::Url) -> Option<ShadowsocksEntry> {
		let raw_username = match base64::engine::general_purpose::STANDARD_NO_PAD.decode(parsed_url.username().as_bytes()) {
			Ok(u) => u,
			Err(e) => {
				println!("[-] Failed to decode username: {}, error: {}", parsed_url.username(), e);
				return None;
			}
		};
		let user_name = match String::from_utf8(raw_username) {
			Ok(u) => u,
			Err(e) => {
				println!("[-] Failed to convert username to string: {}, error: {}", parsed_url.username(), e);
				return None;
			}
		};
		let mut user_name_parts = user_name.splitn(2, ':');
		let cipher = match user_name_parts.next() {
			Some(c) => c.to_string(),
			None => {
				println!("[-] Failed to get cipher from username: {}", user_name);
				return None;
			}
		};
		let password = match user_name_parts.next() {
			Some(p) => p.to_string(),
			None => {
				println!("[-] Failed to get password from username: {}", user_name);
				return None;
			}
		};

		let server = match parsed_url.host_str() {
			Some(s) => s.to_string(),
			None => {
				println!("[-] Failed to get server from uri: {}", parsed_url);
				return None;
			}
		};

		let port = match parsed_url.port() {
			Some(p) => p,
			None => {
				println!("[-] Failed to get port from uri: {}", parsed_url);
				return None;
			}
		};

		let query = match parsed_url.query() {
			Some(q) => q.to_string(),
			None => "".to_string()
		};

		let name = match parsed_url.fragment() {
			Some(f) => urlencoding::decode(f).unwrap().to_string(),
			None => "".to_string()
		};

		Some(ShadowsocksEntry {
			entry_type: "ss".to_string(),
			name,
			server,
			port,
			cipher,
			password,
			plugin: query,
			udp: true
		})
	}
	
}

fn process_sub_link(sub_link: &str) -> Vec<String> {
	println!("[*] Getting sub link: {}", sub_link);
	let res = reqwest::blocking::get(sub_link).expect("Failed to get sub link");
	let body = res.text().expect("Failed to get sub link body");
	let decoded_body = base64::engine::general_purpose::STANDARD.decode(body.as_bytes()).expect("Failed to decode sub link body");
	let decoded_body = String::from_utf8(decoded_body).expect("Failed to convert decoded body to string");
	let url_list = decoded_body.lines().map(|line| line.to_string()).collect::<Vec<String>>();

	url_list
}

// fn generate_clash_config(node_list: &Vec<ProxyNodeEntry>) -> String {
// 	let mut config = String::new();
// 	config.push_str("proxies:\n");
// 	for node in node_list {
// 		match &node.protocol {
// 			ProxyNodeProtocol::Shadowsocks(ss_entry) => {
// 				config.push_str(&format!("  - name: {}\n", ss_entry.name));
// 				config.push_str(&format!("    type: {}\n", ss_entry.entry_type));
// 				config.push_str(&format!("    server: {}\n", ss_entry.server));
// 				config.push_str(&format!("    port: {}\n", ss_entry.port));
// 				config.push_str(&format!("    cipher: {}\n", ss_entry.cipher));
// 				config.push_str(&format!("    password: {}\n", ss_entry.password));
// 				config.push_str(&format!("    udp: {}\n", ss_entry.udp));
// 				config.push_str(&format!("    plugin: {}\n", ss_entry.plugin));
// 			}
// 		}
// 	}
// 	config.push_str("proxy-groups:\n");
// 	config.push_str("  - name: Proxy\n");
// 	config.push_str("    type: select\n");
// 	config.push_str("    proxies:\n");
// 	for node in node_list {
// 		match &node.protocol {
// 			ProxyNodeProtocol::Shadowsocks(ss_entry) => {
// 				config.push_str(&format!("      - {}\n", ss_entry.name));
// 			}
// 		}
// 	}
// 	config.push_str("  - name: Direct\n");
// 	config.push_str("    type: select\n");
// 	config.push_str("    proxies:\n");
// 	config.push_str("      - DIRECT\n");
// 	config.push_str("  - name: Reject\n");
// 	config.push_str("    type: select\n");
// 	config.push_str("    proxies:\n");
// 	config.push_str("      - REJECT\n");
// 	config.push_str("rules:\n");
// 	config.push_str("  - MATCH,Proxy\n");
// 	config.push_str("  - FINAL,Direct\n");
// 	config.push_str("  - MATCH,Reject\n");
// 	config.push_str("  - FINAL\n");

// 	config
// }

fn main() {
    let args: Vec<String> = env::args().collect();
    let sub_link: &str;

    if args.len() == 1 {
        sub_link = "https://dy.tagsub.net/api/v1/client/subscribe?token=801cd9b979676ac84b018debfd57567a";
    } else if args.len() == 2 {
        sub_link = &args[1];
    } else {
        println!("Usage: {} <SUB_LINK>", args[0]);
        return;
    }

	let url_list = process_sub_link(sub_link);

    println!("[*] URI number: {}, parsing...", url_list.len());
	let mut node_list: Vec<ProxyNodeEntry> = Vec::new();
	for u in url_list {
		match ProxyNodeEntry::parse(&u) {
			Some(entry) => node_list.push(entry),
			None => ()
		}
	}

	println!("[*] Generating config...");
	// let config = generate_clash_config(&node_list);
	for node in node_list {
		match node.protocol {
			ProxyNodeProtocol::Shadowsocks(ss_entry) => {
				println!("{}", ss_entry.gen_clash_config());
				return;
			}
		}
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_ss() {
		let ss = "ss://YWVzLTEyOC1nY206aGFNTE1YaXJCeW42ckdWaA@app-zhihu.zhihu.win:20031/?plugin=simple-obfs%3Bobfs%3Dhttp%3Bobfs-host%3D5739b1aa758b.microsoft.com#%F0%9F%87%A6%F0%9F%87%B6%20%E5%8D%97%E6%9E%81%E4%B8%A83x%20AQ";
		// let ss = urlencoding::decode(ss).unwrap();
		// println!("{}", ss);
		
		// let parsed_url = url::Url::parse(&ss).unwrap();

		// // println!("{:?}", parsed_url);
		
		// // // println!("{:?}", base64::engine::general_purpose::STANDARD_NO_PAD.decode(parsed_url.username()).unwrap());
		// // let raw_username = base64::engine::general_purpose::STANDARD_NO_PAD.decode(parsed_url.username().as_bytes()).unwrap();
		// // let user_name = String::from_utf8(raw_username).unwrap();
		// // println!("{}", user_name);

		// let ss_entry = ProxyNodeEntry::to_ss_entry(parsed_url).unwrap();
		// println!("{:?}", ss_entry);
		let ss_entry = ProxyNodeEntry::parse(ss).unwrap();
		println!("{:?}", ss_entry);
	}
}