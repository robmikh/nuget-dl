use nuget_dl::process_nuget;

fn main() {
    let _package_files = process_nuget("examples/nuget.toml").unwrap();
}
