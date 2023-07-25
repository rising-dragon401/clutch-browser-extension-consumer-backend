use refinery::config::Config;
use refinery::config::ConfigDbType;
use refinery::embed_migrations;
use std::fs;
use std::fs::File;
use std::path::Path;

pub fn migrate(url: &str) {
    embed_migrations!("migrations");

    let path = Path::new(url);

    match fs::create_dir_all(path.parent().unwrap()) {
        Ok(dir) => dir,
        Err(e) => panic!("Error creating dir. {}", e),
    };

    if !path.exists() {
        match File::create(path) {
            Ok(file) => file,
            Err(e) => panic!("Error creating file. {}", e),
        };
    }

    let mut c = Config::new(ConfigDbType::Sqlite).set_db_path(url);
    migrations::runner().run(&mut c).unwrap();
}
