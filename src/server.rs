
use super::config::ServerConfig;

use std::{
    io::{prelude::*, BufReader, ErrorKind},
    net::{TcpListener, TcpStream},
    path::Path,
    fs,
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

#[derive(Debug,Clone)]
pub enum HttpStatus {
    Ok=200,
    Forbidden=403,
    NotFound=404,
    InternalServerError=500
}

impl HttpStatus {
    pub fn reason(&self) -> &str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::InternalServerError => "Internal Server Error"
        }
    }
}

impl From<HttpStatus> for u16 {
    fn from(code: HttpStatus) -> u16 {
        code as u16
    }
}
impl TryFrom<u32> for HttpStatus {
    type Error = &'static str;
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(HttpStatus::Ok),
            403 => Ok(HttpStatus::Forbidden),
            404 => Ok(HttpStatus::NotFound),
            500 => Ok(HttpStatus::InternalServerError),
            _ => Err("{value} is not a valid Http Status code!")
        }
    }
}

#[derive(Debug)]
struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub contents: Option<Vec<u8>>
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code: u16 = self.status.clone().into();
        write!(f, "HTTP/1.1 {} {}\r\n",code,self.status.reason())?;
        
        for (key, value) in &self.headers {
            write!(f, "{key}: {value}\r\n")?;
        }
        
        let len = match &self.contents {
            Some(c) => c.len(),
            None => self.body.len()
        };
        write!(f, "Content-Length: {len}\r\n\r\n{}", self.body)

    }
}
#[derive(Debug)]
struct HttpRequest {
    method: String,
    version: String,
    path: String,
    query: String,
    headers: HashMap<String,String>,
    body: Option<String>
}

impl HttpRequest {
    fn parse(tcp_stream: &mut TcpStream) -> Result<Self, ServerError> {
        let buf_reader = BufReader::new(tcp_stream);
        let mut lines = buf_reader.lines();
        
        let (method, url , version): (String, String, String) = match lines.next() {
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

        let url_parts: Vec<&str> = url.split("?").collect(); 
        let path = url_parts[0][1..].into();
        let query= if url_parts.len() == 2 {url_parts[1].into()} else {"".into()}; 

        Ok(HttpRequest { method, version, path, query, headers, body:None}) 
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
                let mut stream = stream.unwrap();
                self.handle_connection(&mut stream);
            }
        }else{
            panic!("Not bound!")
        }
        self
    }

    fn render_directory(&self, path: &Path) -> String {
       let paths = fs::read_dir(path).unwrap();
       let mut members: Vec<String> = vec!["<tr><th>Size</th><th>Name</th></tr>".into()];
        
       let header = if let Some(p) = path.to_str() {
           p.replace(&self.config.root, "root")
       } else {
           "/".into()
       };

       for subpath in paths {
            let p = subpath.unwrap().path();
            let s = p.to_str().unwrap().replace(&self.config.root, "");
            let md= fs::metadata(p).unwrap();

            members.push(
                    format!("<tr><td>{}</td><td><a href='{s}'>{s}{}</a></td></tr>", md.len(), 
                            if md.is_dir() {"/"} else {""}));
       }
       format!("<h1>{header}</h1><br><br><table styling='width:50%'>{}</table>",members.join(""))
    }

    fn handle_connection(&self, stream: &mut TcpStream) {

        let headers: HashMap<String,String> = HashMap::new();
        let request = match HttpRequest::parse(stream) {
            Ok(request) => request,
            Err(_) => {

                let response = HttpResponse { status: HttpStatus::InternalServerError , 
                                                headers,
                                                body:"<h1>500: Internal Server Error</h1>".into(),
                                                contents: None};  

                let resp = format!("{response}");
                stream.write_all(resp.as_bytes()).unwrap();
                return ()
            }
        };

        let req_path = match fs::canonicalize(Path::new(&self.config.root).join(&request.path)) {
            Ok(path) => path,
            Err(err) => { 

                let response = match err.kind() {
                   ErrorKind::NotFound => {
                    HttpResponse {status: HttpStatus::InternalServerError , 
                                                headers,
                                                body:"<h1>404: Not Found</h1>".into(),
                                                contents: None}  
                   },
                   _ => {
                    HttpResponse {status: HttpStatus::InternalServerError , 
                                                headers,
                                                body:"<h1>500: Internal Server Error</h1>".into(),
                                                contents: None}  

                   }
                };
                
                let resp = format!("{response}");
                stream.write_all(resp.as_bytes()).unwrap();
                return ()

            }
        };

        if !req_path.starts_with(&self.config.root) {
                 let response = HttpResponse { status: HttpStatus::Forbidden, 
                                                headers,
                                                body:"<h1>403: Forbidden</h1>".into(),
                                                contents: None};  

                let resp = format!("{response}");
                stream.write_all(resp.as_bytes()).unwrap();
                return ()
           
        }

        if req_path.is_dir() {
            let body = self.render_directory(&req_path);
            let response = HttpResponse { status: HttpStatus::Ok, 
                                                headers,
                                                body,
                                                contents: None};  

            let resp = format!("{response}");
            stream.write_all(resp.as_bytes()).unwrap();

        }else if req_path.is_file() {
            let (body, contents): (String, Option<Vec<u8>>) = match fs::read_to_string(&req_path) {
                Ok(file) => (file, None),
                Err(_) => {
                    let mut f = fs::File::open(&req_path).unwrap();
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer).unwrap();
                    
                    ("".into(), Some(buffer))

                }
            };
           
            let response = HttpResponse { status: HttpStatus::Ok,
                                          headers,
                                          body,
                                          contents:contents.clone()};
            
            let resp = format!("{response}");
            stream.write_all(resp.as_bytes()).unwrap();
            if let Some(cont) = contents {
                stream.write_all(&cont).unwrap();
            }
            stream.flush().unwrap()
            
        }
    }
}


