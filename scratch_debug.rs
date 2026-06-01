fn main() {
    let output = "
* <|ce2324a|2026-05-31|Juan Manuel Amador Roa|Refactor Config
|\\  
| * <|c7e61dc|2026-05-31|Juan|Refactor mouse
* | <|917b2ea|2026-05-31|Juan|Improve code
|/  
* <|e2cef2c|2026-05-24|Juan|Simplify
";
    for line in output.lines().filter(|l| !l.is_empty()) {
        if let Some(idx) = line.find("<|") {
            let graph = &line[..idx];
            let rest = &line[idx + 2..];
            let parts: Vec<&str> = rest.splitn(4, '|').collect();
            if parts.len() == 4 {
                println!("Graph: '{}', Hash: {}, Date: {}, Author: {}, Msg: {}", graph, parts[0], parts[1], parts[2], parts[3]);
            }
        } else {
            println!("Graph only: '{}'", line);
        }
    }
}
