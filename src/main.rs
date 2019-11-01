use std::fs::{ File };
use std::io::{BufReader, Read, BufWriter, Write};
use rustc_serialize::json::{Parser, JsonEvent, StackElement};

fn build_indent(buf: &mut String, indent_lvl: u32) {
    *buf = buf.trim_end().to_string();
    *buf += "\n";
    for _ in 0..indent_lvl {
        *buf += "  ";
    }
}

fn escape_str(wr: &mut dyn std::fmt::Write, v: &str) -> std::fmt::Result {
    wr.write_str("\"")?;
    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => { continue; }
        };

        if start < i {
            wr.write_str(&v[start..i])?;
        }

        wr.write_str(escaped)?;

        start = i + 1;
    }

    if start != v.len() {
        wr.write_str(&v[start..])?;
    }

    wr.write_str("\"")?;
    Ok(())
}

fn json_format(
    parser: &mut Parser<std::str::Chars>,
    buf: &mut String,
    mut indent_lvl: u32)-> std::fmt::Result
{
    let mut dst_str = String::new();
    let mut dst_str_mini = String::new();
    let mut first = true;
    let mut evt: Option<JsonEvent> = parser.next();

    loop {
        if let Some(j) = evt {
            if j != JsonEvent::ArrayEnd && j != JsonEvent::ObjectEnd {
                if !first {
                    dst_str += ", ";
                    dst_str_mini += ", ";
                }
                first = false;
            }
            else {
                indent_lvl = std::cmp::max(indent_lvl - 1, 0);
            }
            build_indent(&mut dst_str, indent_lvl);

            if let Some(stack_top) = parser.stack().top() {
                match stack_top {
                    StackElement::Key(key) => {
                        if j != JsonEvent::ArrayEnd && j != JsonEvent::ObjectEnd {
                            escape_str(&mut dst_str, key)?;
                            escape_str(&mut dst_str_mini, key)?;
                            dst_str += ": ";
                            dst_str_mini += ": ";
                        }
                    },
                    _ =>  {}
                }
            }

            match &j {
                JsonEvent::ObjectStart => {
                    dst_str += "{";
                    dst_str_mini += "{";
                    let mut sub_buf = String::new();
                    json_format(parser, &mut sub_buf, indent_lvl + 1)?;
                    dst_str += &sub_buf;
                    dst_str_mini += &sub_buf;
                },
                JsonEvent::ObjectEnd => {
                    dst_str += "}";
                    dst_str_mini += "}";
                },
                JsonEvent::ArrayStart => {
                    dst_str += "[";
                    dst_str_mini += "[";
                    let mut sub_buf = String::new();
                    json_format(parser, &mut sub_buf, indent_lvl + 1)?;
                    dst_str += &sub_buf;
                    dst_str_mini += &sub_buf;
                },
                JsonEvent::ArrayEnd => {
                    dst_str += "]";
                    dst_str_mini += "]";
                },
                JsonEvent::BooleanValue(v) => {
                    dst_str += if *v { "true" } else { "false" };
                    dst_str_mini += if *v { "true" } else { "false" };
                },
                JsonEvent::F64Value(v) => {
                    dst_str += &format!("{}", v);
                    dst_str_mini += &format!("{}", v);
                },
                JsonEvent::I64Value(v) => {
                    dst_str += &format!("{}", v);
                    dst_str_mini += &format!("{}", v);
                },
                JsonEvent::U64Value(v) => {
                    dst_str += &format!("{}", v);
                    dst_str_mini += &format!("{}", v);
                },
                JsonEvent::StringValue(v) => {
                    escape_str(&mut dst_str, v)?;
                    escape_str(&mut dst_str_mini, v)?;
                    //dst_str += &format!("\"{}\"", v);
                    //dst_str_mini += &format!("\"{}\"", v);
                },
                JsonEvent::NullValue => {
                    dst_str += "null";
                    dst_str_mini += "null";
                },
                _ => {
                    println!("j1:{:?}", j);
                }
            }
            if j == JsonEvent::ArrayEnd || j == JsonEvent::ObjectEnd {
                if dst_str_mini.trim().len() < 80 {
                    *buf += &dst_str_mini;
                }
                else {
                    *buf += &dst_str;
                }
                break;
            }
        }
        else {
            if dst_str_mini.trim().len() < 80 {
                *buf += &dst_str_mini.trim_start();
            }
            else {
                *buf += &dst_str.trim_start();
            }
            break;
        }
        evt = parser.next();
    }
    Ok(())
}

fn usage() {
    let exe_path = std::env::current_exe().unwrap();
    println!("{:?} [json file path]", exe_path.file_name().unwrap());
    println!("→ 渡されたUTF-8エンコードのJSONファイルをいい感じに整形して mod.jsonとして保存します。", )
}

fn main()-> Result<(), std::io::Error> {
    let src_path: &str;
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        src_path = &args[1];
    }
    else {
        usage();
        return Ok(());
    }
    let f = File::open(src_path)?;
    let mut reader = BufReader::new(f);

    let mut j_str = String::new();
    let _ = reader.read_to_string(&mut j_str);
    let mut parser = Parser::new(
        j_str.chars()
    );
    let mut dst_str = String::new();
    let dst = json_format(&mut parser, &mut dst_str, 0);
    if let Err(e) = dst {
        println!("json formatting error: {}", e);
        return Ok(());
    }

    let mut path = std::path::PathBuf::from(src_path);
    let dst_name = &format!("{}_mod.{}",
        path.file_stem().unwrap_or(std::ffi::OsStr::new("")).to_string_lossy(),
        path.extension().unwrap_or(std::ffi::OsStr::new("json")).to_string_lossy());
    path.pop();
    path.push(dst_name);
    let w = File::create(path)?;
    let mut writer = BufWriter::new(w);

    writer.write(dst_str.as_bytes())?;

    return Ok(());
}
