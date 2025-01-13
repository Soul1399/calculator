use std::{fs::File, io::{self, Write}, ops::DerefMut, rc::Rc};

use super::{BracketChunk, BracketConfig, BracketFlag, BracketSection, BracketType, BracketValue, Brackets, CharSlice, ConfigProps, AT_CHAR, CLOSE, OPEN};

impl Brackets {
    pub fn make_indexing(&mut self, config: Option<BracketConfig>) {
        if self.buffer.len() > 0 || self.file_map.is_some() {
            return;
        }
        self.is_processing = true;
        if let Some(c) = config {
            self.config = c;
            if !self.flags.contains(&BracketFlag::HasConfig) {
                self.flags.push(BracketFlag::HasConfig);
            }
        }
        let mut utf8_text: Vec<u8> = Vec::with_capacity(100);
        self.start_index = Some(self.make_config_indexing(&mut utf8_text));
        self.make_values_indexing(&mut utf8_text);

        self.all_open_bks.extend(self.open_bks.iter().map(|x| x.clone()));
        self.all_close_bks.extend(self.close_bks.iter().map(|x| x.clone()));

        self.buffer = String::from_utf8(utf8_text).unwrap();
        self.remove_cache();
        self.is_processing = false;
    }

    pub fn write_into_new_file(&self, full_path: &str) -> io::Result<File> {
        let mut file = File::create_new(full_path)?;
        if let Err(e) = file.write_all(self.buffer.as_bytes()) {
            return Err(e);
        }
        return Ok(file);
    }

    fn make_config_indexing(&mut self, target_bytes: &mut Vec<u8>) -> usize {
        if !self.flags.contains(&BracketFlag::HasConfig) || self.config.is_empty() {
            return 0;
        }
        Self::start_config_into_bytes(target_bytes);
        let mut index: usize = 1;
        self.open_bks.push(BracketChunk { idx: index, is_open_first: Some(false), typ: BracketType::Config, ..Default::default() });
        for p in ConfigProps.iter() {
            let v_len = self.config.get_config_value(p).len();
            if v_len == 0 {continue}
            self.open_bks.push(BracketChunk { idx: index + p.len() + 1, linked_idx: index + p.len() + v_len + 2, is_open_first: Some(false), typ: BracketType::Config, ..Default::default() });
            self.close_bks.push(BracketChunk { idx: index + p.len() + v_len + 2, linked_idx: index + p.len() + 1, typ: BracketType::Config, ..Default::default() });
            index += v_len + p.len() + 3;
            Self::start_value_into_bytes(&p, self.config.get_config_value(p), target_bytes);
            Self::end_value_into_bytes(target_bytes);
        }
        self.close_bks.push(BracketChunk { idx: index, linked_idx: 1, typ: BracketType::Config, ..Default::default() });

        return index + 1;
    }

    fn make_values_indexing(&mut self, target_bytes: &mut Vec<u8>) {
        let temp_root = self.root.take();
        let rc_root = Rc::new(temp_root);
        self.make_value_indexing(self.get_start_index(), rc_root.clone(), target_bytes);
        self.root.replace(Rc::into_inner(rc_root).unwrap());
    }

    fn make_value_indexing(&mut self, index: usize, value: Rc<BracketSection>, target: &mut Vec<u8>) -> usize {
        let mut next_index = index;
        match value.as_ref() {
            BracketSection::Array(a) => {
                let mut temp_array = a.take();
                temp_array.name.start = next_index;
                next_index += Self::start_name_into_bytes(&temp_array.name.value.borrow(), target);
                temp_array.name.end = next_index;
                target.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
                next_index += 1;
                let open_idx = next_index;
                let mut x: usize = 0;
                let ln = temp_array.array.len();
                while x < ln {
                    next_index = self.make_value_indexing(next_index, temp_array.array[x].clone(), target);
                    x += 1;
                }
                temp_array.name.value.replace(Default::default());
                a.replace(temp_array);
                target.extend(CLOSE.encode_utf8(&mut [0u8;4]).as_bytes());
                next_index += 1;
                self.open_bks.push(BracketChunk { is_open_first: Some(self.get_start_index() == index), idx: open_idx, linked_idx: next_index, typ: BracketType::Simple, ..Default::default() });
                self.close_bks.push(BracketChunk { idx: next_index, linked_idx: open_idx, typ: BracketType::Simple, ..Default::default() });
            },
            BracketSection::Real(val) => {
                let mut temp_val = val.take();
                Self::make_final_value_indexing(target, &mut next_index, &mut temp_val);
                val.replace(temp_val);
                next_index += 1;
            },
            BracketSection::Str(val) => {
                Self::make_final_value_indexing(target, &mut next_index, val.borrow_mut().deref_mut());
                next_index += 1;
            }
            BracketSection::Int(val) => {
                Self::make_final_value_indexing(target, &mut next_index, val.borrow_mut().deref_mut());
                next_index += 1;
            },
            _ => { }
        }
        next_index
    }

    fn make_final_value_indexing(target: &mut Vec<u8>, next_index: &mut usize, val: &mut BracketValue) {
        if let BracketType::FreeText(t) = val.btyp {
            *next_index += Self::append_free_text_into_bytes(&t, &val.value.borrow(), target);
        }
        else {
            val.start = *next_index;
            target.extend_from_slice(val.value.borrow().as_bytes());
            val.end = *next_index + val.value.borrow().len();
            *next_index = val.end;
        }
    }

    fn start_config_into_bytes(target: &mut Vec<u8>) {
        // raw approach is faster
        //Self::append_chars_to_utf8(&[AT_CHAR, OPEN], target);
        target.extend(AT_CHAR.encode_utf8(&mut [0u8;4]).as_bytes());
        target.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
    }

    fn start_value_into_bytes(name: &str, value: &str, target: &mut Vec<u8>) {
        Self::start_name_into_bytes(name, target);
        target.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
        target.extend_from_slice(value.as_bytes());
    }

    fn start_name_into_bytes(name: &str, target: &mut Vec<u8>) -> usize {
        let mut total_len = name.len();
        if name.contains(OPEN) || name.contains(CLOSE) {
            total_len += Self::append_escaped_name_into_bytes(name, target);
        }
        else {
            target.extend_from_slice(name.as_bytes());
        }
        total_len
    }

    fn end_value_into_bytes(target: &mut Vec<u8>) {
        //Self::append_chars_to_utf8(&[CLOSE], target);
        target.extend(CLOSE.encode_utf8(&mut [0u8;4]).as_bytes());
    }

    fn append_escaped_name_into_bytes(name: &str, target: &mut Vec<u8>) -> usize {
        // raw approach is faster
        //Self::append_chars_to_utf8(&[OPEN, AT_CHAR], target);
        target.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
        target.extend(AT_CHAR.encode_utf8(&mut [0u8;4]).as_bytes());
        target.push(b' ');
        target.extend(name.as_bytes());
        target.push(b' ');
        target.extend(AT_CHAR.encode_utf8(&mut [0u8;4]).as_bytes());
        target.extend(CLOSE.encode_utf8(&mut [0u8;4]).as_bytes());
        6 + name.len()
    }

    fn append_free_text_into_bytes(slice: &CharSlice, value: &str, target: &mut Vec<u8>) -> usize {
        target.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
        let pfx_sfx_chars = vec![slice.character; slice.quantity].into_boxed_slice();
        Self::append_mixed_to_utf8(&pfx_sfx_chars, &[value], target);
        Self::append_chars_to_utf8(&pfx_sfx_chars, target);
        target.extend(CLOSE.encode_utf8(&mut [0u8;4]).as_bytes());
        (slice.quantity * 2) + 2 + value.len()
    }

    fn append_chars_to_utf8(chars: &[char], target: &mut Vec<u8>) {
        let mut b = [0u8; 4];
        for &c in chars {
            let l = c.encode_utf8(&mut b).len();
            target.extend(&b[..l]);
        }
    }

    fn append_mixed_to_utf8(
        chars: &[char],
        strings: &[&str],
        target: &mut Vec<u8>,
    ) {
        let mut b = [0u8; 4]; // Reusable buffer for chars
    
        // Append chars
        for &c in chars {
            let l = c.encode_utf8(&mut b).len();
            target.extend(&b[..l]);
        }
    
        // Append strings
        for &s in strings {
            target.extend_from_slice(s.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests_brackets_file {
    use std::{fs::{self, File}, io::Read, path::Path, time::Instant};
    use crate::tools::dir::try_move_into_trash;

    use super::*;

    const PATH_FILES: &str = "/home/soul/dev/rust/calculator/src/data";

    #[test]
    fn append_chars_speed() {
        let mut v: Vec<u8> = Vec::with_capacity(100);
        let text = " test of a longer string ";

        v.extend('a'.encode_utf8(&mut [0u8;4]).as_bytes());
        v.clear();
        
        let start = Instant::now();
        Brackets::append_chars_to_utf8(&[OPEN, AT_CHAR], &mut v);
        v.extend(text.as_bytes());
        Brackets::append_chars_to_utf8(&[AT_CHAR, CLOSE], &mut v);
        println!("chars: {:?}", start.elapsed());
        v.clear();

        let start = Instant::now();
        v.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
        v.extend(AT_CHAR.encode_utf8(&mut [0u8;4]).as_bytes());
        v.extend(text.as_bytes());
        v.extend(AT_CHAR.encode_utf8(&mut [0u8;4]).as_bytes());
        v.extend(CLOSE.encode_utf8(&mut [0u8;4]).as_bytes());
        println!("raw: {:?}", start.elapsed());
        v.clear();

        let start = Instant::now();
        Brackets::append_mixed_to_utf8(&[OPEN, AT_CHAR], &[text], &mut v);
        Brackets::append_mixed_to_utf8(&[AT_CHAR, CLOSE], &[], &mut v);
        println!("mixed: {:?}", start.elapsed());
        v.clear();

        let start = Instant::now();
        v.extend(format!("{}{}{}{}{}", OPEN, AT_CHAR, text, AT_CHAR, CLOSE).as_bytes());

        println!("format: {:?}", start.elapsed());

        assert!(true);
    }

    #[test]
    fn test_single_line_file() {
        let filename = Path::new(PATH_FILES).join("products.json");
        let file = File::open(filename.to_str().unwrap());
        if let Ok(mut f) = file {
            let mut buf: String = Default::default();
            _ = f.read_to_string(&mut buf);
            let result = Brackets::build_from_json(&buf);
            assert!(result.is_ok());
            let bk = &mut result.unwrap();
            let root = bk.root.clone();
            if let BracketSection::Array(ref a) = *root.borrow_mut() {
                assert_eq!(a.borrow().array.len(), 4);
                let item = a.borrow().array.iter().next().unwrap().clone();
                if let BracketSection::Array(ref i) = item.as_ref() {
                    assert_eq!(i.borrow().name.value.borrow().as_str(), "products");
                    assert_eq!(i.borrow().array.len(), 30);
                }
            }
            else {
                assert!(false);
            }
            
            bk.make_indexing(None);
            assert_ne!(bk.buffer.len(), 0);
            let pth = Path::new(PATH_FILES).join("products.bk");
            let filename = pth.to_str().unwrap();
            try_move_into_trash(filename);
            let f = bk.write_into_new_file(filename).unwrap();
            assert!(fs::exists(filename).unwrap_or(false));
        }
    }
}