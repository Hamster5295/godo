pub fn get_file_name(tag: &String, mono: &bool) -> String {
    let mut result = format!("Godot_{}", tag);
    if *mono {
        result += "_mono";
    }
    result
}

pub fn get_version_name(tag: &String, mono: &bool) -> String {
    let mut result = format!("Godot {}", tag);
    if *mono {
        result += " mono";
    }
    result
}