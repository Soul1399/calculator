use super::{BracketChunk, BracketConfig, BracketFlag, BracketSection, BracketType, BracketValue, Brackets, CharSlice, ConfigProps, AT_CHAR, CLOSE, OPEN};

impl Brackets {
    fn write_into(&self, buffer: &mut [u8]) {
        
    }

    pub fn make_indexing(&mut self, config: Option<BracketConfig>) {
        if self.buffer.len() > 0 || self.file_map.is_some() {
            return;
        }
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
        let root_value = self.root.clone();
        self.make_value_indexing(self.get_start_index(), &root_value.borrow(), target_bytes);
    }

    fn make_value_indexing(&mut self, index: usize, value: &BracketSection, target: &mut Vec<u8>) -> usize {
        let mut next_index = index;
        match value {
            BracketSection::Array(a) => {
                next_index += Self::start_name_into_bytes(&a.borrow().name.value.borrow(), target);
                target.extend(OPEN.encode_utf8(&mut [0u8;4]).as_bytes());
                next_index += 1;
                let open_idx = next_index;
                for item in a.borrow().array.iter() {
                    next_index = self.make_value_indexing(next_index, &item, target);
                }
                target.extend(CLOSE.encode_utf8(&mut [0u8;4]).as_bytes());
                next_index += 1;
                self.open_bks.push(BracketChunk { is_open_first: Some(self.get_start_index() == index), idx: open_idx, linked_idx: next_index, typ: BracketType::Simple, ..Default::default() });
                self.close_bks.push(BracketChunk { idx: next_index, linked_idx: open_idx, typ: BracketType::Simple, ..Default::default() });
            },
            BracketSection::Real(_) | BracketSection::Int(_) | BracketSection::Str(_) => {
                let val = match value {
                    BracketSection::Real(r) => r.clone(),
                    BracketSection::Int(i) => i.clone(),
                    BracketSection::Str(s) => s.clone(),
                    _ => unreachable!()
                };
                if let BracketType::FreeText(t) = val.borrow().btyp {
                    next_index += Self::append_free_text_into_bytes(&t, &val.borrow().value.borrow(), target);
                }
                else {
                    val.borrow_mut().start = next_index;
                    target.extend_from_slice(val.borrow().value.borrow().as_bytes());
                    val.borrow_mut().end = next_index + val.borrow().value.borrow().len();
                    next_index = val.borrow_mut().end;
                }
                next_index += 1;
            },
            _ => { }
        }
        next_index
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
    use std::time::Instant;

    use super::*;

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
}