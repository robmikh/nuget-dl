use nuget_dl::download_package;

fn main() {
    download_package("WinPixEventRuntime", "1.0.220124001", "").unwrap();
}
