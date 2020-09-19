mod tmp;

fn main() {
    match tmp::initialize_tmp("./tmp") {
        Ok(_) => println!("tmp initialized"),
        Err(e) => eprintln!("tmp initialization error: {}", e),
    }
}
