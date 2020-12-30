// ** Allsorts vs Harfbuzz Test Result **
//
//      Running target/release/deps/allsorts-2eba7236a31af647
// running 1 test
// test allsorts ... bench:      80,779 ns/iter (+/- 238)
//
// test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out; finished in 0.73s
//
//      Running target/release/deps/harfbuzz-40dd6fbf404a596e
//
// running 1 test
// test harfbuzz ... bench:      34,112 ns/iter (+/- 3,517)
//
// test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out; finished in 0.23s
//
//      Running target/release/deps/rustybuzz-45b9c0aee71cc9e6
//
// running 1 test
// test rustybuzz ... bench:      58,906 ns/iter (+/- 7,641)
//
// test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out; finished in 0.47s

#[cfg(test)]
mod tests {
    #[test]
    fn harfbuzz() {
        use harfbuzz_rs::*;
        let path = "/home/ad/hansheng.huang/.local/share/fonts/iosevka-term-regular.ttf";
        let index = 0;
        let face = Face::from_file(path, index).unwrap();
        let font = Font::new(face);
        let features = vec![Feature::new(b"liga", 1, ..), Feature::new(b"calt", 1, ..)];
        let buffer = UnicodeBuffer::new().add_str("\"====>\"");
        let output = shape(&font, buffer, &features);
        let infos = output.get_glyph_infos();
        for info in infos {
            println!("{:?}", info);
        }
    }
}
