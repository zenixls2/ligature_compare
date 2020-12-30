#![feature(test)]
extern crate test;
use rustybuzz::*;
use test::Bencher;
use ttf_parser::Tag;

#[bench]
fn rustybuzz(b: &mut Bencher) {
    let path = "benches/fonts/FiraCode-Regular.ttf";
    let font_data = std::fs::read(path).unwrap();
    let index = 0;
    let face = Face::from_slice(&font_data, index).unwrap();
    let features = vec![
        Feature::new(Tag::from_bytes(&[b'l', b'i', b'g', b'a']), 1, ..),
        Feature::new(Tag::from_bytes(&[b'c', b'a', b'l', b't']), 1, ..),
    ];
    b.iter(|| {
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str("->><--許三蓋!！");
        buffer.reset_clusters();
        let output = shape(&face, &features, buffer);
        output.glyph_infos();
    });
}
