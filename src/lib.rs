use std::{fs::File, path::Path, io::{Write, Read}};

use sha2::{Sha512, Digest};
use xml::{EventReader, reader::XmlEvent};

pub fn download_package_bytes(package_name: &str, version: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let url = format!("https://www.nuget.org/api/v2/package/{package_name}/{version}");
    let bytes: Vec<u8> = reqwest::blocking::get(url)?.bytes()?.iter().map(|x| *x).collect();
    Ok(bytes)
}

pub fn download_package_overwrite<P: AsRef<Path>>(package_name: &str, version: &str, download_dir: P) -> Result<File, Box<dyn std::error::Error>> {
    let download_dir = download_dir.as_ref();
    let bytes = download_package_bytes(package_name, version)?;
    std::fs::create_dir_all(download_dir)?;
    let package_file_name = get_package_file_name(package_name, version);
    let path = {
        let mut path = download_dir.to_owned();
        path.push(package_file_name);
        path
    };
    let file = {
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        file
    };
    Ok(file)
}

pub fn download_package<P: AsRef<Path>>(package_name: &str, version: &str, download_dir: P) -> Result<File, Box<dyn std::error::Error>> {
    let download_dir = download_dir.as_ref();
    
    // Get the download file path
    let package_file_name = get_package_file_name(package_name, version);
    let path = {
        let mut path = download_dir.to_owned();
        path.push(package_file_name);
        path
    };

    // First check if the file is already there
    let matches = if path.exists() {
        package_matches_hash(package_name, version, &path)?
    } else {
        false
    };
    
    let file = if !matches {
        download_package_overwrite(package_name, version, download_dir)?
    } else {
        File::open(&path)?
    };
    Ok(file)
}

fn get_package_file_name(package_name: &str, version: &str) -> String {
    format!("{package_name}.{version}.nupkg")
}

fn package_matches_hash<P: AsRef<Path>>(package_name: &str, version: &str, package_file: P) -> Result<bool, Box<dyn std::error::Error>> {
    // Get the hash from nuget.org
    let hash = get_package_hash(package_name, version)?;
    let reference_hash = base64::decode(&hash.hash)?;

    let mut hasher = match &hash.algorithm {
        HashAlgorithm::SHA512 => Sha512::new(),
        HashAlgorithm::Unknown(_) => {
            // We don't know how to handle this hashing algorithm,
            // assume that it doesn't match.
            return Ok(false);
        }
    };

    // Get the hash from the existing file
    let mut file = File::open(package_file)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    hasher.update(&bytes);
    let file_hash = hasher.finalize();
    
    // Compare the hashes
    let reference_iter = reference_hash.iter();
    let actual_iter = file_hash.iter();
    for (reference, actual) in reference_iter.zip(actual_iter) {
        if *reference != *actual {
            return Ok(false);
        }
    }
    Ok(true)
}

pub struct PackageHash {
    pub hash: String,
    pub algorithm: HashAlgorithm,
}

pub enum HashAlgorithm {
    SHA512,
    Unknown(String),
}

impl HashAlgorithm {
    pub fn from_string(string: String) -> Self {
        match string.as_str() {
            "SHA512" | "sha512" => {
                Self::SHA512
            }
            _ => Self::Unknown(string)
        }
    }
}

pub fn get_package_hash(package_name: &str, version: &str) -> Result<PackageHash, Box<dyn std::error::Error>> {
    let url = format!("https://www.nuget.org/api/v2/Packages(Id='{package_name}',Version='{version}')");
    let text = reqwest::blocking::get(url)?.text()?;

    let parser = EventReader::from_str(&text);
    let mut event_iter = parser.into_iter();
    let mut package_hash = None;
    let mut package_hash_algorithm = None;
    while let Some(e) = event_iter.next() {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                match name.local_name.as_str() {
                    "PackageHash" => {
                        let next_text = get_text(event_iter.next().unwrap().unwrap()).unwrap();
                        package_hash = Some(next_text);
                    }
                    "PackageHashAlgorithm" => {
                        let next_text = get_text(event_iter.next().unwrap().unwrap()).unwrap();
                        package_hash_algorithm = Some(next_text);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }

    let package_hash = package_hash.expect("Package hash not found!");
    let package_hash_algorithm = HashAlgorithm::from_string(package_hash_algorithm.expect("Package hash algorithm not found!"));

    Ok(PackageHash {
        hash: package_hash,
        algorithm: package_hash_algorithm,
    })
}

fn get_text(event: XmlEvent) -> Option<String> {
    match event {
        XmlEvent::Characters(string) => Some(string),
        _ => None
    }
}