use log::{self, Log, Metadata, Record, SetLoggerError, LevelFilter, Level};
use time::{self, OffsetDateTime, format_description, error};
use colored::{Color, Colorize };

// use tools::fs; 

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Mutex;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::ErrorKind;
const DE_TIME_PASE_FORMAT:&str = "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
sign:mandatory]:[offset_minute]:[offset_second]";

/// this can work and be set only before call init();
/// in (Color, Color) first Color and second Color are font color and bg color respectively
/// at the present, We can only use the enum from colored library
#[derive(Debug)]
struct ColorOption {
    use_color: bool,
    level_to_color: HashMap<Level, (Color,Color)>
}

impl ColorOption {//maybe I can extract some method and turn those into a Trait 
    fn new(use_color: bool, level_to_color: HashMap<Level, (Color,Color)>) -> Self {
        ColorOption {
            use_color,
            level_to_color
        }
    }
    fn set_use_color(&mut self, use_color: bool) {
        if use_color != self.use_color {
            self.use_color = use_color;
        }
    }
    fn get_use_color(&self) -> bool {
        self.use_color
    }
    fn get_color_from_level(&self, l: Level) -> (Color,Color) {
        match self.level_to_color.get(&l) {
            Some(c) => *c,
            None => panic!("get_color_from_level error !!!") //未来解决Error问题
        }
    }
}

#[derive(Debug)]
struct TimeOption {
    time_parse_formt: String,
    /// if not utc, the only another one opt. is local_time
    /// default is local_time(true)
    use_local: bool,
    /// default is true
    log_time: bool
}

impl TimeOption {
    fn new() -> Self {
        TimeOption {
            time_parse_formt: String::from(DE_TIME_PASE_FORMAT),
            use_local: true,
            log_time: true
        }
    }
    ///get a formatted time String
    fn get_time(&self) -> Result<String, error::Error>{//time 内置的 Error并没有做到处理
        if self.enabled() {
            let now = if self.use_local {
                OffsetDateTime::now_local()?
            } else {
                OffsetDateTime::now_utc()
            };
            let time_format = format_description::parse(&self.time_parse_formt)?;
            Ok(now.format(&time_format)?)
        } else {
            Ok(String::new())
        }
    }
    fn set_time_format(&mut self, f: &str) {
        self.time_parse_formt = String::from(f);
    }
    fn enabled(&self) -> bool {
        self.log_time
    }
    fn set_use_local(&mut self, is_use: bool) {
        if is_use != self.use_local {
            self.use_local = is_use
        }
    }
    fn set_log_time(&mut self, is_open: bool) {
        if is_open != self.log_time {
            self.log_time = is_open;
        }
    }
    //解决时间生成错误的方案应该是重新生成时间
    fn resolve_time_error(result: Result<String, error::Error>) -> String{//未来返回值可改为一个enum
        match result {
            Ok(t) => t,
            Err(error::Error::IndeterminateOffset(e)) => {
                eprintln!("{e}");
                String::new()
            },
            _ => String::new()
        }
    }
}


///Destination Options
/// in the future, the field in this struct may be changed in a muti-threads pattern
#[derive(Debug)]
struct DestOption {
    ///default: use std out
    use_dest: bool,
    ///this option always combines with file option completing the flush feature
    dest_out: Option<String>,
    // file: Option<File>,
    output_stream: String,
    //第三项我们可以自定义flush，使得外界传入闭包生效
    // custom_fn
}
impl DestOption {
    fn new() -> Self {
        DestOption {
            use_dest: false,
            dest_out: None,
            // file: None,
            output_stream: String::new()
        }
    }
    /// setup with destination file
    fn new_with_dest(dest: &str) -> Self {
        DestOption {
            use_dest: true,
            dest_out: Some(String::from(dest)),
            // file: None,
            output_stream: String::new()
        }
    }
    fn is_use_dest(&self) -> bool {
        self.use_dest
    }
    fn set_use_dest(&mut self, is_open: bool) {
        if self.use_dest != is_open {
            self.use_dest = is_open;
        }
    }
    fn push_output(&mut self, stream: &str) {
        self.output_stream.push_str(stream);
    }
}
#[derive(Debug)]
pub struct ELogger{
    time_option: TimeOption,
    color_option: ColorOption,
    dest: Mutex<RefCell<DestOption>> //use Mutex seems to be a error
}
impl Log for ELogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        let l = record.level(); //Level
        
        let (f_color, _) = self.get_color(l);
        let l = format!("[{l}]"); //String Level

        let time = self.get_time();
        let output = format!("{}\r\n{}\r\n", time, record.args());

        match self.is_use_dest() {
            true => {
                let output = format!("{} {}", l, output);
                self.push_output(&output);
                self.flush();
            },
            false => {
                let l = l.color(f_color);
                let output = format!("{} {}", l, output);
                println!("{output}");
            }
        };
            
    }
    fn flush(&self){
        let guard = self.dest.lock().unwrap();
        match &guard.borrow().dest_out {//only read, don't need to lock
            Some(file_name) => {
                let mut file = open_file(file_name);
                let output = guard.borrow().output_stream.clone();
                if let Ok(_) = file.write(output.as_bytes()) {
                    // file.flush().expect("flush error");
                };
            },
            None => panic!("can't read file without filename")//here we should make a panic 
        };
        guard.borrow_mut().output_stream.clear();
    }
}
fn open_file(file_name: &str) -> File {
    match OpenOptions::new().append(true).open(file_name) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                create_file(file_name) //这里1.需要提示后进行生成2.直接生成
            },
            _ => panic!("unexpected error")
        } 
    }
}

fn create_file(file_name: &str) -> File {
    match OpenOptions::new().create_new(true).append(true).open(file_name) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => panic!("file already exists"),
            _ => panic!("unexpected error")
        }
    }
}
/// you should have done all of initialization setup before calling init();
impl ELogger {
    pub fn new() -> Self {
        //
        let default_level = LevelFilter::Info;
        log::set_max_level(default_level);
        //
        let default_color_map = HashMap::from([
            (Level::Trace, (Color::White, Color::BrightBlack)),
            (Level::Debug, (Color::BrightWhite, Color::Black)),
            (Level::Info, (Color::BrightBlue, Color::Black)),
            (Level::Warn, (Color::BrightYellow, Color::Black)),
            (Level::Error, (Color::BrightRed, Color::Black)),
        ]);
        ELogger{
            time_option: TimeOption::new(),
            color_option: ColorOption::new(true, default_color_map),
            dest: Mutex::new(RefCell::new(DestOption::new()))
        }
    }
    // a shortcut
    pub fn new_dest(dest: &str) -> Self {
        let s = Self::new();
        *s.dest.lock().unwrap() = RefCell::new(DestOption::new_with_dest(dest));
        s
    }
    ///  set the max Level, you can also use log::set_max_level
    ///  but you can only get the max filter level by call log::max_level
    ///  you can only input ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"] case 
    ///  maybe I can cover more one layer upon the log::log! ?
    ///  it's only the old impl, at that time, I want to implement a completely isolation.
    //pub fn set_max_level(self, l: &str) -> Self {
    //let l = match Level::from_str(l) {
    //Ok(l) => l,
    //Err(e) => panic!("you can only input ['ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'] case {e}")
    //};
    //log::set_max_level(l.to_level_filter());
    //self
    //}
    pub fn set_max_level(self, l: Level) -> Self {
        log::set_max_level(l.to_level_filter());
        self
    }

    /// use your custom format to log the time
    /// you can link to time crate <https://crates.io/crates/time> getting the introduction of time format
    pub fn set_time_format(mut self, format: &str) -> Self {
        self.time_option.set_time_format(format);
        self
    }
    /// get the formatted time
    pub fn get_time(&self) -> String {
        TimeOption::resolve_time_error(self.time_option.get_time())
    }
    /// a switch for controlling your log time standard
    pub fn set_use_local(mut self, is_use: bool) -> Self {
        self.time_option.set_use_local(is_use);
        self
    } 
    /// control whether log the time
    pub fn set_log_time(mut self, is_open: bool) -> Self {
        self.time_option.set_log_time(is_open);
        self
    }

    ///enabled to use color format
    pub fn enable_use_color(mut self) -> Self {
        self.color_option.set_use_color(true);
        self
    }
    ///disable to use color format
    pub fn disable_use_color(mut self) -> Self {
        self.color_option.set_use_color(false);
        self
    }
    ///check use_color
    pub fn is_use_color(&self) -> bool {
        self.color_option.get_use_color()
    }
    ///get (Color, Color)
    pub fn get_color(&self, l: Level) -> (Color,Color) {
        self.color_option.get_color_from_level(l)
    }

    /// destination
    pub fn is_use_dest(&self) -> bool {
        self.dest.lock().unwrap().borrow().is_use_dest() //here I haven't resolved the unexpected err
    }
    /// push stream into
    pub fn push_output(&self, stream: &str) {
        self.dest.lock().unwrap().borrow_mut().push_output(stream) // the same as above method
    }
    /// control whether use the
    pub fn set_use_dest(self, is_open: bool) -> Self {
        self.dest.lock().unwrap().borrow_mut().set_use_dest(is_open);
        self
    }
    #[must_use]
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self))
    }
}

///only a logger init without any ohters
pub fn quick_init() -> Result<(), SetLoggerError> {
    ELogger::new().init()
}
///only a init shortcut
pub fn init_use_dest(dest: &str) -> Result<(), SetLoggerError> {
    ELogger::new_dest(dest).init()
}

#[cfg(test)]
mod tests {
    
}
