use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::HashMap;
use std::path::Path;
use std::fs; 

use rem::error::RemError;

/// A structure to store a series of cache operations and a value
/// Cache operations are represented by a single character
pub struct CacheOperation {
    pub commands: Vec<char>,
    pub value: String,
}

impl CacheOperation {
    /// Creates a new CacheOperation instance from an input string
    /// Stores the commands and corresponding value
    /// Write Example: ```W$abc:def```
    /// The resulting command would be W with a value of abc:def
    pub fn new_from_string(cache_op_str: &String) -> CacheOperation {
        let mut cache_op = CacheOperation {
            commands: Vec::new(),
            value: String::new(),
        };
        let mut idx = 0;
        let chars_iter = cache_op_str.chars();
        // Read the string until $ is found
        for c in chars_iter {
            idx += 1;
            if c == '$' {
                // Set the value to everything after $
                cache_op.value = String::from(&cache_op_str[idx..]);
                break;
            }
            // Add a new command for each iteration where $ hasn't been found yet
            cache_op.commands.push(c);
        }
        return cache_op;
    }
}


/// Cache object -- Simple wrapper around a map
#[derive(Debug)]
pub struct Cache {
    pub map_internal: HashMap<String, Vec<u8>>,
}

impl Cache {
    pub fn new() -> Cache {
        return Cache { map_internal: HashMap::new() };
    }

    /// Writes the provided value to the cache using the provided key
    ///
    /// The value will be written to the in memory store and the file store
    ///
    /// The _cache directory will be created if it does not exist
    ///
    /// If a value for the provided key already exists it will be overwritten
    pub fn cache_item(&mut self, key: String, val: Vec<u8>) -> Result<(), RemError> {
        debug!("Writing to cache: key={:?} val={:?}", key, val);
        let dir_res = fs::create_dir("_cache");
        if dir_res.is_err() {
            let err_kind = dir_res.unwrap_err().kind();
            if err_kind == io::ErrorKind::PermissionDenied {
                return Result::Err(RemError::from(io::Error::from(err_kind)));
            }
        }
        let mut f: File = File::create(format!("_cache/{}", key)).unwrap();
        try!(f.write(&val.as_slice()));
        try!(f.flush());
        self.map_internal.insert(key, val);
        return Ok(());
    }

    /// Reads a value from the cache
    ///
    /// If the key is found in the in memory map then the corresponding value is returned
    ///
    /// If the key cannot be found in the map then an attempt will be made to load the value
    /// from the file corresponding with the key
    pub fn read_item(&self, key: String) -> Result<Option<Box<Vec<u8>>>, RemError> {
        let res: Option<Box<Vec<u8>>>;
        if self.map_internal.contains_key(&key) {
            let res_opt = self.map_internal.get(&key);
            match res_opt {
                Some(vec_ref) => res = Some(Box::new(vec_ref.clone())),
                None => res = None,
            }
        } else {
            let mut buf: Vec<u8> = Vec::new();
            match File::open(format!("_cache/{}", key)) {
                Err(_) => res = None,
                Ok(mut file) => {
                    try!(file.read_to_end(&mut buf));
                    res = Some(Box::new(buf));
                }
            };
        }
        return Ok(res);
    }

    /// Delete's an item from the cache
    ///
    /// If the key is found in the in memory map then that entry will be removed
    ///
    /// The file corresponding to the key will also be deleted
    pub fn delete_item(&mut self, key: String) -> Result<(), RemError> {
        if self.map_internal.contains_key(&key) {
            self.map_internal.remove(&key);
        }
        let path = format!("_cache/{}", key);
        if Path::new(&path).exists() {
            match fs::remove_file(path) {
                Ok(x) => x,
                Err(_) => {
                    return Err(RemError::with_reason(format!("Failed to delete file for key: \
                                                                {}",
                                                             key)))
                }
            }
        }
        return Ok(());
    }
}
