use flate2::write::GzEncoder;
use flate2::Compression;
use std::env::args;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, copy};
use std::io::prelude::*;
use subprocess::Exec;

fn main()-> subprocess::Result<()> {

	if args().len() != 3 {
	    eprintln!("Usage: ./morph `program` `process`");
	    Ok(())

	} else {
		let program = args().nth(1).unwrap();
		let process = args().nth(2).unwrap();

		let mut to_open = File::open(&program)?;
	    let mut buffer = Vec::new();
	    to_open.read_to_end(&mut buffer)?;

		create_dir_all(format!("{}_morph/src/", &program))?;
		let data = include_bytes!("template/main.rs");
		let toml = include_bytes!("template/Cargo.toml");

		let mut to_compress = BufReader::new(File::open(&program).unwrap());
	    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
	  
	    copy(&mut to_compress, &mut encoder).unwrap();
	    let compressed = encoder.finish().unwrap();

	    let data_0 = std::str::from_utf8(data).unwrap();
	    let data_1 = data_0.replace("PROGRAM", &program);
	    let data_2 = data_1.replace("PROCESS", &process);
	    let data_3 = data_2.replace("SIZE", &format!("{}", compressed.len() + 32));
	    let toml_0 = std::str::from_utf8(toml).unwrap();
	    let toml_1 = toml_0.replace("PROGRAM", &program);

	    let mut main_file = File::create(format!("{}_morph/src/main.rs", program))?;
	    main_file.write_all(data_3.as_bytes())?;
	    let mut toml_file = File::create(format!("{}_morph/Cargo.toml", program))?;
	    toml_file.write_all(toml_1.as_bytes())?;

	    Exec::shell(format!("cd {}_morph && cargo build --release && cp target/release/{} . && strip {}", &program, &program, &program)).join()?;

	    let mut to_append = OpenOptions::new().write(true).append(true).open(format!("{}_morph/{}", &program, &program)).unwrap();
		let mut urandom = File::open("/dev/urandom")?;
  		let mut key = [0; 32];
  		let mut encrypted = Vec::new();
  		urandom.read(&mut key)?;
  		for i in 0..compressed.len() { encrypted.push(compressed[i] ^ key[i % 32]) }
    	for i in 0..32 { encrypted.push(key[i]) }
		to_append.write_all(&encrypted)?;

		Ok(())
	}
}
