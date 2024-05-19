use snafu::{whatever, ResultExt, Snafu, Whatever};

fn main() -> Result<(), Whatever> {
    is_valid_id(9).whatever_context("ID may not be less than 10")?;

    let path = "non-existent-file.txt";
    read_config_file(path)?;
    Ok(())
}

fn read_config_file(path: &str) -> Result<String, Whatever> {
    std::fs::read_to_string(path).with_whatever_context(|_| format!("Could not read file {path}"))
}

// Turnkey 错误
fn is_valid_id(id: u16) -> Result<(), Whatever> {
    if id < 10 {
        whatever!("ID may not be less than 10, but it was {id}");
    }
    Ok(())
}

// 自定义错误类型 -- 结构体风格
#[derive(Debug, Snafu)]
#[snafu(display("ID may not be less than 10, but it was {id}"))]
pub struct InvalidIdError {
    id: u16,
}

#[allow(unused)]
fn is_valid_id_struct(id: u16) -> Result<(), InvalidIdError> {
    snafu::ensure!(id >= 10, InvalidIdSnafu { id });
    Ok(())
}

// 自定义错误类型 -- 枚举风格
#[allow(unused)]
#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("ID may not be less than 10, but it was {id}"))]
    InvalidIdEnum { id: u16 },
}

#[allow(unused)]
fn is_valid_id_enum(id: u16) -> Result<(), Error> {
    snafu::ensure!(id >= 10, InvalidIdEnumSnafu { id });
    Ok(())
}

//
#[derive(Debug, Snafu)]
#[snafu(display("Could not read file {path}"))]
struct ConfigFileError {
    source: std::io::Error,
    path: String,
}

#[allow(unused)]
fn read_config_file_config(path: &str) -> Result<String, ConfigFileError> {
    std::fs::read_to_string(path).context(ConfigFileSnafu { path })
}
