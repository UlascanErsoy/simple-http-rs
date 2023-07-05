
use super::config::ServerConfig;

use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    collections::HashMap,
    convert::TryFrom,
    fmt
};

pub struct Server {
    pub config: ServerConfig,
    listener: Option<TcpListener>,
}

#[derive(Debug)]
pub enum ServerError {
    RequestParseError,
    HttpStatusError
}

pub enum HttpStatus {
    Ok=200,
    NotFound=404,
    InternalServerError=500
}

impl HttpStatus {
    pub fn reason(&self) -> &str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::InternalServerError => "Internal Server Error"
        }
    }
}

impl TryFrom<u32> for HttpStatus {
    type Error = &'static str;
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(HttpStatus::Ok),
            404 => Ok(HttpStatus::NotFound),
            500 => Ok(HttpStatus::InternalServerError),
            _ => Err("{value} is not a valid Http Status code!")
        }
    }
}

#[derive(Debug)]
struct HttpResponse {
    pub status_code: u32,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP/1.1")
    }
}
#[derive(Debug)]
struct HttpRequest {
    method: String,
    version: String,
    path: String,
    headers: HashMap<String,String>,
    body: Option<String>
}

impl HttpRequest {
    fn parse(mut tcp_stream: TcpStream) -> Result<Self, ServerError> {
        let buf_reader = BufReader::new(&mut tcp_stream);
        let mut lines = buf_reader.lines();
        
        let (method, path, version) = match lines.next() {
            Some(Ok(line)) => {
                let parts = line.split_whitespace().collect::<Vec<&str>>();
                if parts.len() != 3 {
                    return Err(ServerError::RequestParseError);
                }
                
                (parts[0].into(), parts[1].into(), parts[2].into())
            },
            _ => return Err(ServerError::RequestParseError)
        };
        
        let mut headers: HashMap<String, String> = HashMap::new();
        
        for line in lines {
            if let Ok(l) = line {
                if l.is_empty() {
                    break;
                }
                let parts: Vec<&str> = l.split(":").collect();

                if parts.len() == 2 {
                    headers.insert(parts[0].into(), parts[1].into());
                }
            }
        }
        Ok(HttpRequest { method, version, path, headers, body:None}) 
    }
}

impl Server {
    pub fn new(config: ServerConfig) -> Server {
        Server { config , listener: None }
    }

    pub fn bind(&mut self) -> &mut Self {
        let host = format!("{}:{}", self.config.host, self.config.port);

         self.listener = match TcpListener::bind(&host) {
             Ok(l) => Some(l),
             Err(e) => panic!("Could not bind to {host}! {e}")
         };
        
        self 
    }

    pub fn listen(&self) -> &Self {
        
        println!("Listening to {} on port {}",self.config.host, self.config.port);

        if let Some(listener) = &self.listener {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                self.handle_connection(stream);
            }
        }else{
            panic!("Not bound!")
        }
        self
    }

    fn handle_connection(&self, stream: TcpStream) {
        let request = match HttpRequest::parse(stream) {
            Ok(request) => request,
            Err(_) => return ()
        };

        
        
        println!("{:?}", request);
    }
}


