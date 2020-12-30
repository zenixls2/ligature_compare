#![feature(test)]
extern crate test;
use allsorts::binary::read::ReadScope;
use allsorts::font::read_cmap_subtable;
use allsorts::gpos::*;
use allsorts::gsub::*;
use allsorts::gsub::{self, GsubFeatureMask};
use allsorts::layout::*;
use allsorts::tables::{cmap::*, *};
use allsorts::{tag, DOTTED_CIRCLE};
use anyhow::{anyhow, Result};
use test::Bencher;
use tinyvec::*;

struct LoadedFont<'a> {
    cmap_subtable: CmapSubtable<'a>,
    _gpos_cache: LayoutCache<GPOS>,
    gsub_cache: LayoutCache<GSUB>,
    gdef_table: Option<GDEFTable>,
    num_glyphs: u16,
    dotted_index: u16,
}

impl<'a> LoadedFont<'a> {
    fn load_font(data: &'a [u8]) -> Self {
        let scope = ReadScope::new(data);
        let file = scope.read::<OpenTypeFont>().unwrap();
        let otf = match file.data {
            OpenTypeData::Single(ref v) => v,
            _ => panic!(),
        };
        let cmap = otf
            .read_table(&file.scope, tag::CMAP)
            .unwrap()
            .unwrap()
            .read::<Cmap>()
            .unwrap();
        let (_, cmap_subtable): (_, CmapSubtable<'a>) = read_cmap_subtable(&cmap)
            .unwrap()
            .ok_or_else(|| anyhow!("CMAP subtable not found"))
            .unwrap();

        let maxp = otf
            .read_table(&file.scope, tag::MAXP)
            .unwrap()
            .unwrap()
            .read::<MaxpTable>()
            .unwrap();
        let num_glyphs = maxp.num_glyphs;

        let gsub_table = otf
            .find_table_record(tag::GSUB)
            .unwrap()
            .read_table(&file.scope)
            .unwrap()
            .read::<LayoutTable<GSUB>>()
            .unwrap();
        let gdef_table: Option<GDEFTable> = otf
            .find_table_record(tag::GDEF)
            .map(|gdef_record| -> Result<GDEFTable> {
                Ok(gdef_record
                    .read_table(&file.scope)
                    .unwrap()
                    .read::<GDEFTable>()
                    .unwrap())
            })
            .transpose()
            .unwrap();
        let _gpos_cache = otf
            .find_table_record(tag::GPOS)
            .map(|gpos_record| {
                new_layout_cache(
                    gpos_record
                        .read_table(&file.scope)
                        .unwrap()
                        .read::<LayoutTable<GPOS>>()
                        .unwrap(),
                )
            })
            .unwrap();
        let gsub_cache = new_layout_cache(gsub_table);
        let dotted_index = cmap_subtable
            .map_glyph(DOTTED_CIRCLE as u32)
            .unwrap()
            .unwrap_or(0);

        Self {
            cmap_subtable,
            _gpos_cache,
            gsub_cache,
            gdef_table,
            num_glyphs,
            dotted_index,
        }
    }

    fn make_glyph(&self, ch: char) -> Result<Option<RawGlyph<()>>> {
        if let Some(glyph_index) = self
            .cmap_subtable
            .map_glyph(ch as u32)
            .map_err(|e| anyhow!("Error while looking up glyph {}: {}", ch, e))?
        {
            Ok(Some(RawGlyph {
                unicodes: tiny_vec![[char; 1] => ch],
                glyph_index,
                liga_component_pos: 0,
                glyph_origin: GlyphOrigin::Char(ch),
                small_caps: false,
                multi_subst_dup: false,
                is_vert_alt: false,
                fake_bold: false,
                fake_italic: false,
                extra_data: (),
                variation: None,
            }))
        } else {
            Ok(None)
        }
    }
    pub fn shape_text<T: AsRef<str>>(
        &self,
        text: T,
        script: u32,
        lang: Option<u32>,
    ) -> Result<Vec<Info>> {
        let opt_glyphs: Result<Vec<_>> = text
            .as_ref()
            .chars()
            .map(|ch| self.make_glyph(ch))
            .collect();
        let mut glyphs: Vec<_> = opt_glyphs?.into_iter().flatten().collect();

        gsub::apply(
            self.dotted_index,
            &self.gsub_cache,
            self.gdef_table.as_ref(),
            script,
            lang,
            &Features::Mask(GsubFeatureMask::LIGA | GsubFeatureMask::CALT),
            self.num_glyphs,
            &mut glyphs,
        )
        .unwrap();
        // init_from_glyphs elides entries that have no glyph in current font.
        let infos = Info::init_from_glyphs(self.gdef_table.as_ref(), glyphs);
        // skip counting position
        /*if let Some(gpos_cache) = self._gpos_cache.as_ref() {
            let kerning = true;
            gpos_apply(
                gpos_cache,
                self.gdef_table.as_ref(),
                kerning,
                script,
                lang,
                &mut infos,
            )?;
        }*/
        Ok(infos)
    }
}

#[bench]
fn allsorts(b: &mut Bencher) {
    let path = "benches/fonts/FiraCode-Regular.ttf";
    let data = std::fs::read(path).unwrap();
    let font = LoadedFont::load_font(&data);
    let script = tag::from_string("DFLT").unwrap();
    let lang = Some(tag::from_string("dflt").unwrap());
    b.iter(|| {
        let text = "->><--許三蓋!！";
        font.shape_text(text, script, lang).unwrap();
    });
    //let text = "->><--許三蓋!！";
    //let output = font.shape_text(text, script, lang).unwrap();
    //for o in output {
    //    println!("{:?}", o);
    //}
}
