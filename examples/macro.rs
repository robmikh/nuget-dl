use nuget_dl::nuget_packages;

fn main() {
    let _package_files = nuget_packages! {
        { "WinPixEventRuntime", "1.0.220124001" },
        { "Microsoft.AI.DirectML", "1.9.0" },
        { "directxtk12_desktop_2019", "2022.7.30.1" },
    }
    .unwrap();
}
