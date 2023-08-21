use clap::Parser;
use config::{config_file, init, root_dir, unless_exists, write_config};
use javaup::{config, request_builder, Args, CommandType};

// https://github.com/foojayio/discoapi

fn main() {
    unless_exists(&root_dir(), init);
    let args = Args::parse();
    match args.command {
        CommandType::Install(jdkinfo) => request_builder::RequestBuilder::new()
            .install(jdkinfo)
            .execute(),
        CommandType::List { listtype } => request_builder::RequestBuilder::new()
            .list(listtype)
            .execute(),
        CommandType::Default(jdk) => {
            let mut config = config_file();
            config.default_jdk = Some(jdk);
            write_config(&config)
        }
    }
}
