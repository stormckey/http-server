use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "The request cli")]
enum Args {
    #[structopt(about = "Set a key-value pair")]
    Set {
        #[structopt(help = "Key")]
        key: String,
        #[structopt(help = "Value")]
        value: String,
    },
    Del {
        #[structopt(help = "Key")]
        key: String,
    },
    #[structopt(about = "Get the value for a key")]
    Get {
        #[structopt(help = "Key")]
        key: String,
    },
    Ping {},
}

#[tokio::main]
async fn main(){
    let args = Args::from_args();
    let url = "http://127.0.0.1:3000/".to_owned() + &(match args{
        Args::Set {key, value} => {"set?key=".to_owned() + &key + "&value=" +&value},
        Args::Get {key} => {"get/".to_owned() + &key},
        Args::Del {key} => {"del/".to_owned() + &key},
        Args::Ping {} => {"ping".to_owned()},
    });
    let res = reqwest::get(url).await;
    match res {
        Ok(res) => {
            println!("Status: {}", res.status());
            println!("Body:\n{}", res.text().await.unwrap());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
