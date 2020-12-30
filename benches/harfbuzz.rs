#![feature(test)]
extern crate test;
use harfbuzz_rs::*;
use test::Bencher;

#[bench]
fn harfbuzz(b: &mut Bencher) {
    let path = "benches/fonts/FiraCode-Regular.ttf";
    let index = 0;
    let face = Face::from_file(path, index).unwrap();
    let font = Font::new(face);
    let features = vec![
        Feature::new(Tag::new('l', 'i', 'g', 'a'), 1, ..),
        Feature::new(Tag::new('c', 'a', 'l', 't'), 1, ..),
    ];
    b.iter(|| {
        let buffer = UnicodeBuffer::new().add_str("->><--許三蓋!！");
        let output = shape(&font, buffer, &features);
        output.get_glyph_infos();
    });
    let buffer = UnicodeBuffer::new().add_str("ct sp st");
    let output = shape(&font, buffer, &features);
    let infos = output.get_glyph_infos();
    for o in infos {
        println!("{:?}", o);
    }
}
