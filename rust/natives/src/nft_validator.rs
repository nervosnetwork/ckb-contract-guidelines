extern crate alloc;

#[path = "../../contracts/nft-validator/src/validator.rs"]
mod validator;

fn main() {
    if let Err(err) = validator::validate() {
        std::process::exit(err as i32);
    }
}
