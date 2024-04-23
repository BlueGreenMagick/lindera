use std::{fs, io::Write, path::Path};

use lindera_dictionary_builder::{
    build_user_dictionary, CharDefBuilderOptions, CostMatrixBuilderOptions, DictBuilderOptions,
    UnkBuilderOptions, UserDictBuilderOptions,
};

#[cfg(feature = "compress")]
use lindera_compress::compress;
use lindera_core::{
    character_definition::CharacterDefinitions, dictionary::UserDictionary,
    dictionary_builder::DictionaryBuilder, error::LinderaErrorKind, LinderaResult,
};
use lindera_decompress::Algorithm;

const SIMPLE_USERDIC_FIELDS_NUM: usize = 3;
const SIMPLE_WORD_COST: i16 = -10000;
const SIMPLE_CONTEXT_ID: u16 = 0;
const DETAILED_USERDIC_FIELDS_NUM: usize = 13;
const COMPRESS_ALGORITHM: Algorithm = Algorithm::Deflate;

pub struct IpadicBuilder {}

impl IpadicBuilder {
    const UNK_FIELDS_NUM: usize = 11;

    pub fn new() -> Self {
        IpadicBuilder {}
    }
}

impl Default for IpadicBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryBuilder for IpadicBuilder {
    fn build_dictionary(&self, input_dir: &Path, output_dir: &Path) -> LinderaResult<()> {
        fs::create_dir_all(output_dir)
            .map_err(|err| LinderaErrorKind::Io.with_error(anyhow::anyhow!(err)))?;

        let chardef = self.build_chardef(input_dir, output_dir)?;
        self.build_unk(input_dir, &chardef, output_dir)?;
        self.build_dict(input_dir, output_dir)?;
        self.build_cost_matrix(input_dir, output_dir)?;

        Ok(())
    }

    fn build_user_dictionary(&self, input_file: &Path, output_file: &Path) -> LinderaResult<()> {
        let user_dict = self.build_user_dict(input_file)?;
        build_user_dictionary(user_dict, output_file)
    }

    fn build_chardef(
        &self,
        input_dir: &Path,
        output_dir: &Path,
    ) -> LinderaResult<CharacterDefinitions> {
        CharDefBuilderOptions::default()
            .encoding("EUC-JP")
            .compress_algorithm(COMPRESS_ALGORITHM)
            .builder()
            .unwrap()
            .build(input_dir, output_dir)
    }

    fn build_unk(
        &self,
        input_dir: &Path,
        chardef: &CharacterDefinitions,
        output_dir: &Path,
    ) -> LinderaResult<()> {
        UnkBuilderOptions::default()
            .encoding("EUC-JP")
            .compress_algorithm(COMPRESS_ALGORITHM)
            .unk_fields_num(Self::UNK_FIELDS_NUM)
            .builder()
            .unwrap()
            .build(input_dir, chardef, output_dir)
    }

    fn build_dict(&self, input_dir: &Path, output_dir: &Path) -> LinderaResult<()> {
        DictBuilderOptions::default()
            .flexible_csv(false)
            .encoding("EUC-JP")
            .compress_algorithm(COMPRESS_ALGORITHM)
            .normalize_details(true)
            .skip_invalid_cost_or_id(false)
            .builder()
            .unwrap()
            .build(input_dir, output_dir)
    }

    fn build_cost_matrix(&self, input_dir: &Path, output_dir: &Path) -> LinderaResult<()> {
        let matrix_data_path = input_dir.join("matrix.def");
        CostMatrixBuilderOptions::default()
            .encoding("EUC-JP")
            .compress_algorithm(COMPRESS_ALGORITHM)
            .builder()
            .unwrap()
            .build(&matrix_data_path, output_dir)
    }

    fn build_user_dict(&self, input_file: &Path) -> LinderaResult<UserDictionary> {
        UserDictBuilderOptions::default()
            .simple_userdic_fields_num(SIMPLE_USERDIC_FIELDS_NUM)
            .detailed_userdic_fields_num(DETAILED_USERDIC_FIELDS_NUM)
            .simple_word_cost(SIMPLE_WORD_COST)
            .simple_context_id(SIMPLE_CONTEXT_ID)
            .flexible_csv(true)
            .simple_userdic_details_handler(Box::new(|row| {
                Ok(vec![
                    row[1].to_string(), // POS
                    "*".to_string(),    // POS subcategory 1
                    "*".to_string(),    // POS subcategory 2
                    "*".to_string(),    // POS subcategory 3
                    "*".to_string(),    // Conjugation type
                    "*".to_string(),    // Conjugation form
                    row[0].to_string(), // Base form
                    row[2].to_string(), // Reading
                    "*".to_string(),    // Pronunciation
                ])
            }))
            .builder()
            .unwrap()
            .build(input_file)
    }
}

#[cfg(feature = "compress")]
fn compress_write<W: Write>(
    buffer: &[u8],
    algorithm: Algorithm,
    writer: &mut W,
) -> LinderaResult<()> {
    let compressed = compress(buffer, algorithm)
        .map_err(|err| LinderaErrorKind::Compress.with_error(anyhow::anyhow!(err)))?;
    bincode::serialize_into(writer, &compressed)
        .map_err(|err| LinderaErrorKind::Io.with_error(anyhow::anyhow!(err)))?;

    Ok(())
}

#[cfg(not(feature = "compress"))]
fn compress_write<W: Write>(
    buffer: &[u8],
    _algorithm: Algorithm,
    writer: &mut W,
) -> LinderaResult<()> {
    writer
        .write_all(buffer)
        .map_err(|err| LinderaErrorKind::Io.with_error(anyhow::anyhow!(err)))?;

    Ok(())
}
