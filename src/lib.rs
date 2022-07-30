use std::{fs::File, path::Path, io::{BufWriter, Write}};

pub fn download_package_bytes(package_name: &str, version: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let url = format!("https://www.nuget.org/api/v2/package/{}/{}", package_name, version);
    let bytes: Vec<u8> = reqwest::blocking::get(url)?.bytes()?.iter().map(|x| *x).collect();
    Ok(bytes)
}

pub fn download_package<P: AsRef<Path>>(package_name: &str, version: &str, download_dir: P) -> Result<File, Box<dyn std::error::Error>> {
    let download_dir = download_dir.as_ref();
    let bytes = download_package_bytes(package_name, version)?;
    std::fs::create_dir_all(download_dir)?;
    let package_file_name = format!("{}.{}.nupkg", package_name, version);
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
