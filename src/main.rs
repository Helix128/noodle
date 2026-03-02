use console::style;

mod ndl;
use ndl::http::HttpListener;
use ndl::debug::log;
use ndl::files;
fn main() { 

    log::init();

    let version = env!("CARGO_PKG_VERSION");
    println!("{} {} {}{}","Running",style("Noodle").bold(), style("v").green(), style(version).green());
    
    files::preprocess();    
    
    let listener = HttpListener::new(80);
    listener.start();
}
    