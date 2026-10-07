// Append to the bottom of your file.
//
// Cargo.toml:
// [dev-dependencies]
// tempfile = "3"
// image = { version = "0.25", default-features = false, features = ["png", "jpeg", "webp"] }

#[cfg(test)]
mod tests {

    use crate::config::ImageFormat;
    use crate::files::prelude::*;

    use image::{DynamicImage, ImageFormat as ImgFmt};
    use std::collections::BTreeSet;
    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // ---------- helpers ----------

    /// Encode a tiny real image so img_parts has valid containers to parse.
    fn encoded(fmt: ImgFmt) -> Vec<u8> {
        let img = DynamicImage::new_rgb8(2, 2);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), fmt).unwrap();
        buf
    }

    fn names(paths: Vec<PathBuf>) -> BTreeSet<String> {
        paths
            .iter()
            .map(|ee| file_name_with_extension(ee))
            .collect()
    }

    const URL: &str = "https://example.com/page?a=1&b=2";

    // ---------- human_bytes ----------

    #[test]
    fn human_bytes_small_values() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1), "1 B");
        assert_eq!(human_bytes(1023), "1023 B");
    }

    #[test]
    fn human_bytes_scales_units() {
        assert_eq!(human_bytes(1024), "1.0 KB");
        assert_eq!(human_bytes(1536), "1.5 KB");
        assert_eq!(human_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(human_bytes(1024u64.pow(3)), "1.0 GB");
        assert_eq!(human_bytes(1024u64.pow(4)), "1.0 TB");
        assert_eq!(human_bytes(1024u64.pow(5)), "1.0 PB");
    }

    #[test]
    fn human_bytes_caps_at_pb() {
        // u64::MAX is ~16384 PB; must not index past UNITS
        assert!(human_bytes(u64::MAX).ends_with(" PB"));
    }

    // ---------- file name helpers ----------

    #[test]
    fn file_name_helpers() {
        let p = PathBuf::from("/tmp/dir/shot.final.png");
        assert_eq!(file_name_with_extension(&p), "shot.final.png");
        assert_eq!(file_name_without_extension(&p), "shot.final");
    }

    #[test]
    fn file_name_helpers_degenerate_paths() {
        let p = PathBuf::from("/");
        assert_eq!(file_name_with_extension(&p), "");
        assert_eq!(file_name_without_extension(&p), "");
    }

    // ---------- rename_file ----------

    #[test]
    fn rename_file_keeps_extension() {
        let dir = tempdir().unwrap();
        let old = dir.path().join("a.png");
        fs::write(&old, b"x").unwrap();

        let new = rename_file(&old, "b").unwrap();

        assert_eq!(new, dir.path().join("b.png"));
        assert!(new.exists());
        assert!(!old.exists());
    }

    #[test]
    fn rename_file_refuses_to_overwrite() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();

        assert!(rename_file(&a, "b").is_err());
        assert_eq!(fs::read(&a).unwrap(), b"a");
        assert_eq!(fs::read(&b).unwrap(), b"b");
    }

    #[test]
    fn rename_file_missing_source_errors() {
        let dir = tempdir().unwrap();
        assert!(rename_file(&dir.path().join("nope.png"), "x").is_err());
    }

    #[test]
    #[ignore = "known bug: files without an extension get a trailing dot ('name.')"]
    fn rename_file_without_extension_has_no_trailing_dot() {
        let dir = tempdir().unwrap();
        let old = dir.path().join("README");
        fs::write(&old, b"x").unwrap();
        let new = rename_file(&old, "NOTES").unwrap();
        assert_eq!(new, dir.path().join("NOTES"));
    }

    // ---------- last_modified ----------

    #[test]
    fn last_modified_ok_and_err() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("f");
        fs::write(&f, b"x").unwrap();
        assert!(last_modified(&f).is_ok());
        assert!(last_modified(&dir.path().join("missing")).is_err());
    }

    // ---------- embed / read capture URL ----------

    #[test]
    fn roundtrip_png() {
        let out = embed_capture_url(encoded(ImgFmt::Png), ImageFormat::Png, URL).unwrap();
        assert_eq!(read_capture_url(&out).as_deref(), Some(URL));
        // still a decodable image
        image::load_from_memory(&out).unwrap();
    }

    #[test]
    fn roundtrip_jpeg() {
        let out = embed_capture_url(encoded(ImgFmt::Jpeg), ImageFormat::Jpeg, URL).unwrap();
        assert_eq!(read_capture_url(&out).as_deref(), Some(URL));
        image::load_from_memory(&out).unwrap();
    }

    #[test]
    fn roundtrip_webp() {
        let out = embed_capture_url(encoded(ImgFmt::WebP), ImageFormat::Webp, URL).unwrap();
        assert_eq!(read_capture_url(&out).as_deref(), Some(URL));
    }

    #[test]
    fn read_capture_url_none_without_metadata() {
        assert_eq!(read_capture_url(&encoded(ImgFmt::Png)), None);
        assert_eq!(read_capture_url(&encoded(ImgFmt::Jpeg)), None);
        assert_eq!(read_capture_url(&encoded(ImgFmt::WebP)), None);
    }

    #[test]
    fn read_capture_url_none_for_garbage() {
        assert_eq!(read_capture_url(&[]), None);
        assert_eq!(read_capture_url(b"hello world, not an image"), None);
        // right magic, truncated body
        assert_eq!(read_capture_url(&[0x89, b'P', b'N', b'G']), None);
        assert_eq!(read_capture_url(&[0xFF, 0xD8]), None);
    }

    #[test]
    fn embed_rejects_mismatched_format() {
        // JPEG bytes told to parse as PNG must error, not panic
        assert!(embed_capture_url(encoded(ImgFmt::Jpeg), ImageFormat::Png, URL).is_err());
    }

    #[test]
    fn embed_twice_still_readable() {
        let once = embed_capture_url(encoded(ImgFmt::Png), ImageFormat::Png, "first").unwrap();
        let twice = embed_capture_url(once, ImageFormat::Png, "second").unwrap();
        // read_capture_url returns the first matching chunk
        assert_eq!(read_capture_url(&twice).as_deref(), Some("first"));
    }

    // ---------- has_image_header ----------

    #[test]
    fn has_image_header_detects_formats() {
        let dir = tempdir().unwrap();
        let cases: Vec<(&str, Vec<u8>, bool)> = vec![
            ("a.jpg", vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 0], true),
            (
                "a.png",
                vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
                true,
            ),
            ("a.gif", b"GIF89a....".to_vec(), true),
            ("b.gif", b"GIF87a....".to_vec(), true),
            ("a.webp", b"RIFF\0\0\0\0WEBPVP8 ".to_vec(), true),
            ("riff.webp", b"RIFF\0\0\0\0WAVEfmt ".to_vec(), false),
            ("text.png", b"just some text".to_vec(), false),
            ("empty.png", vec![], false),
            ("short.webp", b"RIFF".to_vec(), false),
        ];
        for (name, bytes, expected) in cases {
            let p = dir.path().join(name);
            fs::write(&p, bytes).unwrap();
            assert_eq!(has_image_header(&p), expected, "{name}");
        }
    }

    #[test]
    fn has_image_header_missing_file() {
        let dir = tempdir().unwrap();
        assert!(!has_image_header(&dir.path().join("missing.png")));
    }

    // ---------- glob_to_regex / glob_matches ----------

    #[test]
    fn glob_to_regex_output() {
        assert_eq!(glob_to_regex("*.log"), r"(?i)^[^/]*\.log$");
        assert_eq!(glob_to_regex("**"), "(?i)^.*$");
        assert_eq!(glob_to_regex("a?c"), "(?i)^a[^/]c$");
    }

    #[test]
    fn glob_single_star_does_not_cross_slash() {
        assert!(glob_matches("*.log", "a.log"));
        assert!(!glob_matches("*.log", "dir/a.log"));
    }

    #[test]
    fn glob_double_star_crosses_slash() {
        assert!(glob_matches("src/**", "src/a/b/c.rs"));
        assert!(glob_matches("**/*.log", "dir/sub/a.log"));
    }

    #[test]
    fn glob_question_mark() {
        assert!(glob_matches("a?c", "abc"));
        assert!(!glob_matches("a?c", "a/c"));
        assert!(!glob_matches("a?c", "ac"));
    }

    #[test]
    fn glob_escapes_regex_metacharacters() {
        assert!(glob_matches("a.b", "a.b"));
        assert!(!glob_matches("a.b", "axb"));
        assert!(glob_matches("file(1).png", "file(1).png"));
        assert!(glob_matches("a+b", "a+b"));
        assert!(!glob_matches("a+b", "aab"));
        assert!(glob_matches("[x]", "[x]"));
        assert!(!glob_matches("[x]", "x"));
    }

    #[test]
    fn glob_is_case_insensitive_and_anchored() {
        assert!(glob_matches("FOO", "foo"));
        assert!(!glob_matches("foo", "foobar"));
        assert!(!glob_matches("foo", "xfoo"));
    }

    #[test]
    fn glob_strips_leading_and_trailing_slashes() {
        assert!(glob_matches("/build/", "build"));
        assert!(glob_matches("build/", "build"));
    }

    #[test]
    fn glob_empty_patterns_never_match() {
        assert!(!glob_matches("", "anything"));
        assert!(!glob_matches("/", "anything"));
        assert!(!glob_matches("//", ""));
    }

    // ---------- is_ignored ----------

    #[test]
    fn is_ignored_matches_full_path_or_name() {
        let pats = vec!["node_modules".to_string(), "*.log".to_string()];
        assert!(is_ignored(Path::new("node_modules"), &pats));
        assert!(is_ignored(Path::new("src/node_modules"), &pats)); // via file name
        assert!(is_ignored(Path::new("logs/deep/a.log"), &pats)); // via file name
        assert!(!is_ignored(Path::new("src/main.rs"), &pats));
    }

    #[test]
    fn is_ignored_no_patterns() {
        assert!(!is_ignored(Path::new("anything"), &[]));
    }

    // ---------- load_ignore_patterns ----------

    #[test]
    fn load_ignore_patterns_none() {
        let dir = tempdir().unwrap();
        assert!(load_ignore_patterns(dir.path()).is_empty());
    }

    #[test]
    fn load_ignore_patterns_skips_comments_and_blanks_and_trims() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join(".gitignore"),
            "# comment\n\n  target  \n*.log\n   \n",
        )
        .unwrap();
        assert_eq!(load_ignore_patterns(dir.path()), vec!["target", "*.log"]);
    }

    #[test]
    fn load_ignore_patterns_prefers_dockeepignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "from_git\n").unwrap();
        fs::write(dir.path().join(".dockeepignore"), "from_dockeep\n").unwrap();
        assert_eq!(load_ignore_patterns(dir.path()), vec!["from_dockeep"]);
    }

    // ---------- read_dir ----------

    #[test]
    fn read_dir_lists_files_recursively_not_dirs() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub/deeper")).unwrap();
        fs::write(dir.path().join("a.txt"), b"").unwrap();
        fs::write(dir.path().join("sub/b.txt"), b"").unwrap();
        fs::write(dir.path().join("sub/deeper/c.txt"), b"").unwrap();

        let got = names(read_dir(dir.path()));
        assert_eq!(
            got,
            ["a.txt", "b.txt", "c.txt"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        );
    }

    #[test]
    fn read_dir_prunes_ignored_directories() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("skip")).unwrap();
        fs::write(dir.path().join("skip/hidden.png"), b"").unwrap();
        fs::write(dir.path().join("keep.png"), b"").unwrap();
        fs::write(dir.path().join(".dockeepignore"), "skip\n").unwrap();

        let got = names(read_dir(dir.path()));
        assert!(got.contains("keep.png"));
        assert!(!got.contains("hidden.png"));
    }

    #[test]
    fn read_dir_ignores_by_extension_pattern() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.png"), b"").unwrap();
        fs::write(dir.path().join("b.tmp"), b"").unwrap();
        fs::write(dir.path().join(".gitignore"), "*.tmp\n").unwrap();

        let got = names(read_dir(dir.path()));
        assert!(got.contains("a.png"));
        assert!(!got.contains("b.tmp"));
    }

    #[test]
    fn read_dir_nonexistent_root_is_empty() {
        let dir = tempdir().unwrap();
        assert!(read_dir(&dir.path().join("nope")).is_empty());
    }

    // ---------- read_images ----------

    #[test]
    fn read_images_filters_by_extension_and_header() {
        let dir = tempdir().unwrap();
        let p = dir.path();
        fs::write(p.join("real.png"), encoded(ImgFmt::Png)).unwrap();
        fs::write(p.join("real.jpg"), encoded(ImgFmt::Jpeg)).unwrap();
        fs::write(p.join("real.jpeg"), encoded(ImgFmt::Jpeg)).unwrap();
        fs::write(p.join("real.webp"), encoded(ImgFmt::WebP)).unwrap();
        fs::write(p.join("UPPER.PNG"), encoded(ImgFmt::Png)).unwrap(); // case-insensitive ext
        fs::write(p.join("fake.png"), b"not an image").unwrap(); // bad header
        fs::write(p.join("notes.txt"), b"hello").unwrap(); // bad ext
        fs::write(p.join("anim.gif"), b"GIF89a....").unwrap(); // header ok, ext not allowed
        fs::write(p.join("noext"), encoded(ImgFmt::Png)).unwrap(); // no ext

        let got = names(read_images(p));
        let want: BTreeSet<String> = [
            "real.png",
            "real.jpg",
            "real.jpeg",
            "real.webp",
            "UPPER.PNG",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(got, want);
    }

    #[test]
    fn read_images_respects_ignore_file() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("cache")).unwrap();
        fs::write(dir.path().join("cache/x.png"), encoded(ImgFmt::Png)).unwrap();
        fs::write(dir.path().join("y.png"), encoded(ImgFmt::Png)).unwrap();
        fs::write(dir.path().join(".dockeepignore"), "cache\n").unwrap();

        let got = names(read_images(dir.path()));
        assert_eq!(got, ["y.png".to_string()].into_iter().collect());
    }
}
