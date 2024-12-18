use std::fmt::format;
use std::fs::{self, File};
use std::sync::Arc;

use structopt::StructOpt;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use chrono::{DateTime, Local, Utc};

// use tokio::time::{sleep, Duration};
// use tokio::io::{self, AsyncWriteExt};

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "tcp-listen", about = "Simple TCP/HTTP Server usage.")]
struct Opt {
    /// listen port,
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short = "p", long = "port", default_value = "8080")]
    port: u64,

    /// sleep (ms) per request
    #[structopt(short = "s", long, default_value = "0")]
    sleep: u64,

    /// request level tcp or http (ms)
    #[structopt(short = "l", long, default_value = "7")]
    level: u8,

    /// response content path, default "-->OK!"
    #[structopt(short = "r", long, default_value = "")]
    response_path: String,

     /// no log?
     #[structopt(short = "n", long)]
     nolog: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let mut rvalue = "-->OK!\r\n".to_string();
    if opt.response_path != "" {
        let p = std::path::Path::new(&opt.response_path);
        if !p.is_file() {
            println!("`path` is Not a File.");
            std::process::exit(1);
        }
        // let input = File::open(p).unwrap();
        // let buffered = std::io::BufReader::new(input);
        rvalue = fs::read_to_string(opt.response_path).unwrap();
    }
    if opt.level != 4 && opt.level != 7 {
        std::process::exit(1);
    }
    let nolog= opt.nolog;

    let mut return_value = rvalue;
    let mut notfound = "".to_string();
    if opt.level == 7 {
        let content_type = format!("Content-Type: text/html;charset=utf-8{}", CRLF);
        let server = format!("Server: Rust{}", CRLF);
        let content_length = format!("Content-Length: {}{}", return_value.as_bytes().len(), CRLF);
        // let response: &str = "";
        let status200: String = status_fn(200, "OK");
        return_value = format!(
            "{0}{1}{2}{3}{4}{5}",
            status200, server, content_type, content_length, CRLF, return_value
        );
        notfound = format!(
            "{0}{1}{2}{3}{4}",
            status_fn(404, "NOT FOUND"),
            server,
            content_type,
            format!("Content-Length: {}{}", 0, CRLF),
            CRLF
        );
    }
    println!("每个请求延时时间:{}ms", opt.sleep);
    println!("四层or七层？: {}", opt.level);
    // println!("请求响应内容路径:%s\n", path);
    println!("监听端口：{}", opt.port);
    println!("响应内容：\n{}", return_value);
    println!(
        "结束位: {}, ps: 最后一个字符为十进制，如果使用jmeter tcp压测，请设置为",
        return_value.as_bytes().last().unwrap()
    );

    /*  begin req count server */
    use tokio::sync::mpsc;
    let (count_sender, mut count_receiver) = mpsc::unbounded_channel::<u8>();
    let count_task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(1000));
        let mut count = 0;
        let mut close_count = 0;
        // let mut duration = tokio::time::Duration::from_millis(0);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    // let now = tokio::time::Instant::now();
                    let current_local: DateTime<Local> = Local::now();
                    let custom_format = current_local.format("%Y-%m-%d %H:%M:%S");
                    if !nolog {
                        println!("[{}] req {} r/s, close {:?}", custom_format, count, close_count,);
                    }
                    // if count + err_count != 0 {
                    //     // now("");
                    // } else {
                    //     println!("[{:?}] req {:?} r/s, close {:?}", custom_format, count, err_count,);
                    // }
                    count = 0;
                    close_count = 0;
                }
                val = count_receiver.recv() => {
                    match val.unwrap() {
                        1 => {
                            count += 1;
                            // println!("{}", val.unwrap());
                        }
                        _ => {
                            close_count += 1;
                        }
                    }
                    // let d = val.unwrap();
                }
            }
        }
    });

    let return_value_arc = Arc::new(return_value);
    let notfound_arc = Arc::new(notfound);
    // Bind the listener to the address
    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", opt.port))
        .await
        .unwrap();
    println!("[{}] lisenting...", opt.port);
    loop {
        let count_sender_clone = count_sender.clone();
        // The second item contains the ip and port of the new connection.
        let (stream, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        let opt_sleep = opt.sleep;
        let network_level = opt.level;
        let new200 = Arc::clone(&return_value_arc);
        let newnot = Arc::clone(&notfound_arc);
        // 1. 静态、全局变量
        // 2. struct，解决错误 value moved here, in previous iteration of loop
        tokio::spawn(async move {
            if let Ok(_) = handle_f(
                stream,
                opt_sleep,
                network_level,
                &new200,
                &newnot,
                count_sender_clone,
            )
            .await
            {
                // println!("----out")
            }
        });
    }
    count_task.await.unwrap();

}

fn test(n: u64, b: &str) {
    println!("{}{}", n, b);
}

const CRLF: &str = "\r\n";
fn status_fn(code: i32, text: &str) -> String {
    format!("HTTP/1.1 {} {}{}", code, text, CRLF)
}

async fn handle_f(
    mut stream: TcpStream,
    opt_sleep: u64,
    network_level: u8,
    response: &str,
    notfound: &str,
    sender: tokio::sync::mpsc::UnboundedSender<u8>,
) -> io::Result<()> {
    loop {
        let mut buffer = [0; 1500];
        let size = stream.read(&mut buffer).await?;
        if size == 0 {
            // println!("{} 连接已关闭", stream.peer_addr()?);
            sender.send(0).unwrap();
            return Ok(());
        }
        // println!("{}", std::str::from_utf8( &buffer[..size]).unwrap() );
        if opt_sleep != 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(opt_sleep)).await;
        }
        if network_level == 4 {
            stream.write(response.as_bytes()).await?;
        } else {
            //  if network_level == 7
            if buffer.starts_with(b"GET / HTTP/1") {
                stream.write(response.as_bytes()).await?;
            } else {
                stream.write(notfound.as_bytes()).await?;
            };
        }
        stream.flush().await?;
        sender.send(1).unwrap();
    }
    return Ok(());
}
