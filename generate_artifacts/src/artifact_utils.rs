use std::{
    fs::{self, File},
    io::Read,
};

use serde::{de::DeserializeOwned, Serialize};

fn get_file_as_byte_vec(filename: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}

pub fn get_or_generate_file<T>(path: &str, generating_function: &dyn Fn() -> T) -> T
where
    T: DeserializeOwned + Serialize,
{
    let buffer = get_file_as_byte_vec(path);
    let object;
    if buffer.is_ok() {
        return serde_json::from_slice::<T>(&buffer.unwrap()).unwrap();
    } else {
        object = generating_function();
        let data = serde_json::to_string(&object).unwrap();
        let folder_path = path.rsplit_once("/").unwrap().0;
        fs::create_dir_all(folder_path).unwrap();
        fs::write(path, data).expect("Unable to write file");
    }
    object
}
