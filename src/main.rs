extern crate clap;
mod task;
use std::error::Error;

#[tokio::main]
async fn main() ->Result<(),Box<dyn Error>> {
    //task::run_executor();
    task::run_block();
    //println!("Hello, world!\n");
    Ok(())
}
