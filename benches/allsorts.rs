#![feature(test)]
extern crate test;
use test::Bencher;
use allsorts::gsub::*;
use allsorts::binary::read::{ReadScope, ReadScopeOwned};
use allsorts::font_data_impl::{read_cmap_subtable, Encoding};
use allsorts::gpos::*;
use allsorts::layout::*;
use allsorts::tables::{*, cmap::*};
use allsorts::gsub::GsubFeatureMask;
use allsorts::tag;
use allsorts::unicode::VariationSelector;
use anyhow::{anyhow, Result};
use tinyvec::*;

struct LoadedFont {
    cmap_subtable: CmapSubtable<'static>,
    _gpos_cache: Option<LayoutCache<GPOS>>,
    gsub_cache: LayoutCache<GSUB>,
    gdef_table: Option<GDEFTable>,
    _hmtx: HmtxTable<'static>,
    _hhea: HheaTable,
    num_glyphs: u16,
    _units_per_em: u16,
    _scope: ReadScopeOwned,
}

impl LoadedFont {
    fn load_font<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        let owned_scope = ReadScopeOwned::new(ReadScope::new(&data));
        let file: OpenTypeFile<'static> = unsafe { std::mem::transmute(owned_scope.scope().read::<OpenTypeFile>()?) };
        let otf = match &file.font {
            OpenTypeFont::Single(v) => v,
            _ => panic!(),
        };
        let head =otf
            .read_table(&file.scope, tag::HEAD)?
            .ok_or_else(|| anyhow!("HEAD table missing or broken"))?
            .read::<HeadTable>()?;
        let cmap = otf
            .read_table(&file.scope, tag::CMAP)?
            .ok_or_else(|| anyhow!("CMAP table missing or broken"))?
            .read::<Cmap>()?;
        let (encoding, cmap_subtable): (Encoding, CmapSubtable<'static>) =
            read_cmap_subtable(&cmap)?.ok_or_else(|| anyhow!("CMAP subtable not found"))?;

        assert!(matches!(encoding, Encoding::Unicode));

        let maxp = otf
            .read_table(&file.scope, tag::MAXP)?
            .ok_or_else(|| anyhow!("MAXP table not found"))?
            .read::<MaxpTable>()?;
        let num_glyphs = maxp.num_glyphs;

        let _hhea = otf
            .read_table(&file.scope, tag::HHEA)?
            .ok_or_else(|| anyhow!("HHEA table not found"))?
            .read::<HheaTable>()?;
        let _hmtx = otf
            .read_table(&file.scope, tag::HMTX)?
            .ok_or_else(|| anyhow!("HMTX table not found"))?
            .read_dep::<HmtxTable>((
                usize::from(maxp.num_glyphs),
                usize::from(_hhea.num_h_metrics),
            ))?;

        let gsub_table = otf
            .find_table_record(tag::GSUB)
            .ok_or_else(|| anyhow!("GSUB table record not found"))?
            .read_table(&file.scope)?
            .read::<LayoutTable<GSUB>>()?;
        let gdef_table: Option<GDEFTable> = otf
            .find_table_record(tag::GDEF)
            .map(|gdef_record| -> Result<GDEFTable> {
                Ok(gdef_record.read_table(&file.scope)?.read::<GDEFTable>()?)
            })
            .transpose()?;
        let opt_gpos_table = otf
            .find_table_record(tag::GPOS)
            .map(|gpos_record| -> Result<LayoutTable<GPOS>> {
                Ok(gpos_record
                    .read_table(&file.scope)?
                    .read::<LayoutTable<GPOS>>()?)
            })
            .transpose()?;
        let gsub_cache = new_layout_cache(gsub_table);
        let _gpos_cache = opt_gpos_table.map(new_layout_cache);

        Ok(Self {
            cmap_subtable,
            _hmtx,
            _hhea,
            _gpos_cache,
            gsub_cache,
            gdef_table,
            num_glyphs,
            _units_per_em: head.units_per_em,
            _scope: owned_scope,
        })
    }

    fn glyph_index_for_char(&self, c: char) -> Result<Option<u16>> {
        self.cmap_subtable
            .map_glyph(c as u32)
            .map_err(|e| anyhow!("Error while looking up glyph {}: {}", c, e))
    }
    pub fn shape_text<T: AsRef<str>>(&self, text: T, script :u32, lang: u32) -> Result<Vec<Info>> {
        let mut glyphs = vec![];
        for c in text.as_ref().chars() {
            glyphs.push(RawGlyph {
                unicodes: tiny_vec!([char; 1], c),
                glyph_index: self.glyph_index_for_char(c)?,
                liga_component_pos: 0,
                glyph_origin: GlyphOrigin::Char(c),
                small_caps: false,
                multi_subst_dup: false,
                is_vert_alt: false,
                fake_bold: false,
                fake_italic: false,
                variation: Some(VariationSelector::VS15),
                extra_data: (),
            });
        }

        gsub_apply_default(
            &|| vec![],
            &self.gsub_cache,
            self.gdef_table.as_ref(),
            script,
            lang,
            GsubFeatureMask::LIGA | GsubFeatureMask::CALT,
            self.num_glyphs,
            &mut glyphs,
        )?;
        // init_from_glyphs elides entries that have no glyph in current font.
        let infos = Info::init_from_glyphs(self.gdef_table.as_ref(), glyphs)?;
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
    let font = LoadedFont::load_font(path).unwrap();
    let script = tag::from_string("DFLT").unwrap();
    let lang = tag::from_string("dflt").unwrap();
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
