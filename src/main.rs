extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate glob;

use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use actix_web::{App, HttpRequest, server, middleware, fs, Result};
use actix_web::http::Method;
use glob::glob;


fn index(_req: HttpRequest) -> &'static str {
    "iPXE server listens here. For the actual script see <a href=\"/ipxe\">here</a>. \
    This is a service which basically reads the locations of iso files from the iso \
    directory, then generates a bootable iPXE script from it."
}

fn serve_file(req: HttpRequest) -> Result<fs::NamedFile> {
    let path: PathBuf = PathBuf::from(format!("iso/{}", req.match_info().get("name").unwrap()));
    Ok(fs::NamedFile::open(path)?)
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("hello-world");

    server::new(
        || App::new()
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.f(index))
            .resource("/ipxe", |r| r.f(ipxe_script_gen))
            .resource("/file/{name}", |r| r.f(serve_file)))
        .bind("0.0.0.0:8080").unwrap()
        .start();

    println!("Started http server: 0.0.0.0:8080");
    let _ = sys.run();
}

fn ipxe_script_gen(_req: HttpRequest) -> String {
    let mut ipxe_boot: String = String::new();
    let mut files: Vec<PathBuf> = vec![];
    let mut files2: Vec<PathBuf> = vec![];

    for entry in glob("iso/**/*.iso").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                files.push(path.to_owned());
                files2.push(path.to_owned());
                },
            Err(e) => println!("{:?}", e),
        }
    }

    ipxe_boot.push_str("#!ipxe\necho Boot Menu\nset menu-default isoselect\nmenu isoselect\n");

    for file in files {
        ipxe_boot.push_str(format!("item {} {} \n", 
            file.file_name().unwrap_or(OsStr::new("default.iso")).to_str().unwrap_or("default.iso"), 
            file.strip_prefix("iso").unwrap_or(Path::new("default.iso")).display()).as_str());
    }

    ipxe_boot.push_str("\nchoose os && goto ${os}\n\n");

    for file in files2 {
        ipxe_boot.push_str(&format!(":{} \n", 
            file.file_name().unwrap_or(OsStr::new("default.iso")).to_str().unwrap_or("default.iso")));
        ipxe_boot.push_str(format!("sanboot http://10.10.0.10:8080/file/{}\n\n", 
            file.strip_prefix("iso").unwrap_or(Path::new("default.iso")).display()).as_str());
    } 

    ipxe_boot
}
