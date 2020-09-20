mod iops;

fn main() {
    match iops::initialize_tmp("./tmp") {
        Ok(_) => println!("tmp initialized"),
        Err(e) => eprintln!("tmp initialization error: {}", e),
    }

    match iops::initialize_vm("vm1") {
        Ok(_) => println!("vm initialized"),
        Err(e) => eprintln!("vm initialization error: {}", e),
    }

    match iops::terminate_vm("vm1") {
        Ok(_) => println!("vm terminated"),
        Err(e) => eprintln!("vm termination error: {}", e),
    }
}
