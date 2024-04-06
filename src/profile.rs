// [[file:../zotero.note::*imports][imports:1]]
use gut::prelude::*;
// imports:1 ends here

// [[file:../zotero.note::*core][core:1]]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Find the path to zotero preference file `prefs.js`
fn get_zotero_profile_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let f = Path::new(&home).join(".zotero/zotero/profiles.ini");
    debug!("reading zotero ini profile: {}", f.display());

    for (sec, prop) in ini::Ini::load_from_file(&f).ok()?.iter() {
        if sec.unwrap().starts_with("Profile") {
            let d: HashMap<&str, &str> = prop.iter().collect();
            if let Some(v) = d.get("Default") {
                if *v == "1" {
                    return d.get("Path").map(|x| f.with_file_name(x).join("prefs.js"));
                }
            }
        }
    }
    None
}

// user_pref("extensions.zotero.dataDir", "/home/ybyygu/Data/zotero");
fn parse_zotero_data_dir_from_pref_js(s: &str) -> Option<PathBuf> {
    for line in s.lines() {
        if line.contains("extensions.zotero.dataDir") {
            for x in line.rsplit("\"") {
                return Some(PathBuf::from(x));
            }
        }
    }
    None
}

/// Locate zotero data dir from preference
pub(crate) fn guess_zotero_data_dir() -> Option<PathBuf> {
    let pref_js = get_zotero_profile_path()?;

    let f = std::fs::read_to_string(&pref_js).ok()?;
    parse_zotero_data_dir_from_pref_js(&f)
}

#[test]
fn test_zotero_profile() {
    let pref_js = get_zotero_profile_path().unwrap();
    assert!(pref_js.exists());

    assert!(guess_zotero_data_dir().is_some());
}
// core:1 ends here

// [[file:../zotero.note::bc5986f8][bc5986f8]]
/// Update zotero db file to a cached location when it has been updated.
pub fn update_zotero_db_cache(dbfile: &Path, cached: &Path) -> Result<()> {
    let t1 = std::fs::metadata(dbfile)?.modified()?;
    if !cached.exists() {
        // create leading directory
        let parent_dir = cached.parent().unwrap();
        if !parent_dir.exists() {
            info!("creating leading dir: {:?}", parent_dir);
            std::fs::create_dir_all(parent_dir)?;
        }
        copy_to(dbfile, cached)?;
    } else {
        let t2 = std::fs::metadata(cached)?.modified()?;
        if t2 < t1 {
            info!("zotero db source has been updated.");
            copy_to(dbfile, cached)?;
        }
    }

    Ok(())
}

fn copy_to(src: &Path, dest: &Path) -> Result<()> {
    info!("copy {:?} to {:?}", src, dest);
    let _ = std::fs::copy(src, dest)?;
    Ok(())
}
// bc5986f8 ends here

// [[file:../zotero.note::*test][test:1]]
#[test]
#[ignore]
fn test_db_cache() -> Result<()> {
    let dbfile: &Path = "/home/ybyygu/Data/zotero/zotero.sqlite".as_ref();
    let cached: &Path = "/home/ybyygu/.cache/zotero.sqlite".as_ref();
    update_zotero_db_cache(dbfile, cached)?;
    Ok(())
}
// test:1 ends here
