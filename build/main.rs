use std::path::PathBuf;

use winreg::{
	enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
	RegKey,
};

const STEAM_PATHS: &[&str] = &[
	"SOFTWARE\\WOW6432Node\\Valve\\Steam",
	"SOFTWARE\\Valve\\Steam",
];

const HKEYS: &[RegKey] = &[
	RegKey::predef(HKEY_LOCAL_MACHINE),
	RegKey::predef(HKEY_CURRENT_USER),
];

fn steam_install_dir() -> Option<PathBuf> {
	STEAM_PATHS
		.iter()
		.find_map(|path| {
			HKEYS.iter().find_map(|regkey| {
				regkey
					.open_subkey(path)
					.map(|path| path.get_value::<String, &str>("InstallPath"))
					.map(core::result::Result::ok)
					.ok()
					.flatten()
			})
		})
		.map(PathBuf::from)
}

pub fn gmod_dir() -> Option<PathBuf> {
	let gmod_dir = steam_install_dir()?
		.join("steamapps")
		.join("common")
		.join("GarrysMod");

	if gmod_dir.exists() {
		Some(gmod_dir)
	} else {
		None
	}
}

fn main() {
	let gmod = gmod_dir()
		.expect("Couldn't find your garrysmod install.");

	#[cfg(target_arch = "x86_64")]
	let lua_shared = {
		let x = gmod.join("bin/win64/lua_shared.dll");
		if x.exists() {
			Some(x)
		} else {
			let x = gmod.join("bin/lua_shared.dll");
			if x.exists() {
				Some(x)
			} else {
				None
			}
		}
	};

	#[cfg(not(target_arch = "x86_64"))]
	let lua_shared = {
		let x = gmod.join("bin/lua_shared.dll");
		if x.exists() {
			Some(x)
		} else {
			let x = gmod.join("bin/win64/lua_shared.dll");
			if x.exists() {
				Some(x)
			} else {
				None
			}
		}
	};

	let lua_shared = lua_shared.expect("Couldn't find lua_shared.dll");
	println!("cargo:rustc-link-search=native={}", concat!(env!("CARGO_MANIFEST_DIR"), "/build"));
	println!("cargo:rustc-link-lib=dylib={}", lua_shared.file_stem().unwrap().to_string_lossy());
}