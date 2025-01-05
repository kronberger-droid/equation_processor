use equation_processor::*;
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::fs::File;

#[test]
fn test_csv_parsing() {
    let csv_content = "active,body,name\nyes,x = y + z,example_equation\nno,E = mc^2,\n";
    let path = PathBuf::from("./tests/sample.csv");
    fs::create_dir_all("./tests").unwrap();
    let mut file = File::create(&path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    let equations = read_csv_file(&path).unwrap();
    assert_eq!(equations.len(), 2);
    assert_eq!(equations[0].name, "example_equation");
    assert_eq!(equations[1].name, "default_equation");

    fs::remove_file(path).unwrap();
}

#[test]
fn test_markdown_parsing() {
    let md_content = "%%yes%%\n$$x = y + z$$\n%%example_equation%%\n";
    let path = PathBuf::from("./tests/sample.md");
    fs::create_dir_all("./tests").unwrap();
    let mut file = File::create(&path).unwrap();
    file.write_all(md_content.as_bytes()).unwrap();

    let content = read_file(&path).unwrap();
    let equations = parse_markdown(&content);
    assert_eq!(equations.len(), 1);
    assert_eq!(equations[0].name, "example_equation");

    fs::remove_file(path).unwrap();
}
