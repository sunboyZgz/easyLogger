/*
 * @Author: sunboy
 * @LastEditors: sunboy
 * @Date: 2022-06-04 19:15:29
 * @LastEditTime: 2022-06-08 13:35:17
 */
#![allow(unused_must_use)]
use easy_logger::*;
use log::*;
#[test]
fn test_log() {
  ELogger::new().init();
  log!(Level::Info,"initialization completed");
}
#[test]
#[ignore]
fn test_chain_call() {
  ELogger::new().set_max_level(Level::Trace).init();
  log!(Level::Info,"initialization completed");
}
#[test]
#[ignore]
fn test_color() {
  ELogger::new().set_max_level(Level::Trace).init();
  log!(Level::Trace,"initialization completed");
  log!(Level::Debug,"initialization completed");
  log!(Level::Info,"initialization completed");
  log!(Level::Info,"initialization completed");
  log!(Level::Warn,"initialization completed");
  log!(Level::Error,"initialization completed");
}

#[test]
fn test_time_format() {
  let time_format = "[year]-[month]-[day] [hour]:[minute]:[second]";
  ELogger::new()
  .set_max_level(Level::Trace)
  .set_time_format(time_format)
  .set_use_local(false)
  .init();
  log!(Level::Info,"initialization completed");
}

#[test]
fn test_module() {
  println!(module_path!());
  println!(file!());
  println!("{}",line!());
}

#[test]
fn test_flush() {
  init_use_dest("log.txt");
  log::set_max_level(Level::Trace.to_level_filter());
  log!(Level::Trace,"initialization completed");
  debug!("initialization completed1");
  info!("initialization completed2");
  warn!("initialization completed3");
  error!("initialization completed4");
}