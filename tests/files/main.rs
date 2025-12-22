fn task_one(input: &[String]) -> String {
    wrapper(input[0].to_string())
}

fn task_two(input: &[String]) -> String {
    wrapper(input[1].to_string())
}

fn wrapper(s: String) -> String {
    if std::env::var("DAY").is_ok() {
        let path = std::env::current_dir().unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        format!("{}-{}", s, name)
    } else {
        s
    }
}

fn main() {
    let input = read_input(get_input_file());
    println!("{}", task_one(&input));
    println!("{}", task_two(&input));
}

fn read_input<P>(path: P) -> Vec<String>
where
    P: AsRef<std::path::Path>,
{
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn get_input_file() -> String {
    std::env::args()
        .nth(1)
        .unwrap_or_else(|| "input".to_string())
}
