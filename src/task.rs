pub mod dowloadfile;
pub mod blockRangeIter;
mod threadpool;
//use async_std::fs::File;
//use async_std::io::ReadExt;
//use async_std::io::prelude::WriteExt;
use std::path::PathBuf;
//use async_std::path::PathBuf;
use std::{fs::File,fs::OpenOptions,io::Write};
//use dowloadfile::Dowloadfile;
use reqwest::{ header,Error,header::HeaderValue};
use url::Url;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};
use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};
use std::collections::HashSet;
use std::path::Path;
use tokio::runtime::Runtime;
//use std::convert::TryInto;
use indicatif::{ProgressBar, ProgressStyle};
use clap::{App,Arg};
use std::str::FromStr;
use crate::task::dowloadfile::Dowloadfile;
use crate::task::blockRangeIter::Range;

/// # Examples
/// ```rust
/// mod task;
/// fn main() {
///     Task::run_block();
/// }
/// 
pub fn run_block(){
    let (spawner,executor) = new_spawner();
    
    spawner.spawn( async {
        //const CHUNK_SIZE:u32 = 1024*1024*2;
        let client = reqwest::Client::new();
        let mut output_dir="";
        let argparse = App::new("downfast")
                    .about("downfast 应用为快速下载而生")
                    .version("0.1.0")
                    .author("gogojames")
                    .arg(Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .help("保存文件到本地的路径")
                    )
                    .arg(Arg::with_name("url")
                        .index(1).required(true)
                        .help("运行方式$./downfast http://host/file")
                    )
                    .get_matches();
        let url = argparse.value_of("url").unwrap();
        let out = argparse.value_of("output").unwrap_or("./");
        let contents = client
                            .get(url)
                            .send().await.unwrap();
        let headers = contents.headers().clone();
        let mut filename = Task::get_filename(url, &headers);
        std::fs::create_dir_all(out).unwrap_or_else(|y| {
           println!("! {:?}", y.kind());
        });
        if Path::new(out).is_dir() {
            output_dir = out;
        }else{
            filename = out;
        }
        
        let mut output_filename = PathBuf::from(output_dir);
        output_filename.push(filename);
        let path = output_filename.as_path();
       // let new_filename:String;
       /*  if path.exists() {
            //new_filename =
            output_filename = PathBuf::from(output_dir);
            output_filename.push(new_filename);
            path = output_filename.as_path();
        } */
        //let mut output_file = match OpenOptions::new().create(true).append(true).open(&path){
        
         let fullfile_str = match path.to_str(){
             Some(s)=>s,
             None=>"",
         };
         println!("{}",fullfile_str);
        match Task::new(url,fullfile_str).await {
                            Ok(())=>{
                                0
                             // t.block_down(url,&mut output_file).await
                                  
                                  // t.block_down(url,&mut output_file).await
                            },
                            Err(e)=>{
                                println!("error: {}",e);
                                0
                            }
                        }
                    });
        drop(spawner);
        
        executor.run();
}

fn filename_from_headers(headers:&header::HeaderMap) -> &str {
    let mut ret="";
    if let Some(cdisp) = headers.get(header::CONTENT_DISPOSITION) {
        let mut cdtype:Vec<&str> = cdisp.to_str().unwrap().split(';').collect();
        let set:HashSet<_> =["inline".to_string(),"attachment".to_string()].iter().cloned().collect();
        if cdtype.len()>1 && set.contains(&cdtype[0].trim().to_lowercase()) {
            cdtype.retain(|&val| val.trim().starts_with("filename="));
            if cdtype.len() == 1 {
                let filenames:Vec<&str> = cdtype[0].split('=').collect();
                let filename = filenames[1].trim();
                if let Some(base_filename) = Path::new(filename).file_name() {
                    ret = &base_filename.to_str().unwrap();
                }
            }
        }
    }
    ret
}

fn filename_from_url(url:&str) -> &str {
    let url_path = Path::new(url);
    url_path.file_name().unwrap().to_str().unwrap()
}

struct SharedState {
    completed:bool,
    waker:Option<Waker>,
}

pub struct Task{
    lower:u64,
    upper:u64,
    share_state: Arc<Mutex<SharedState>>,
}

impl Future for Task {
    type Output = u64;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut share_state = self.share_state.lock().unwrap();
        if share_state.completed {
            println!("ready.");
            Poll::Ready(self.upper)
        } else {
            println!("not ready.");
            share_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl  Task{
    pub async fn new<'a>(url:&'a str,fullfile_str:&'a str)->Result<(),Error>{
       // let pool = threadpool::ThreadPool::new(1);
       /* if let Some((last,_)) =pars.split_last() {
           file_name = format!("{}",last).as_str();
       } */
       let share_state=Arc::new(Mutex::new(SharedState{completed:false,waker:None}));
       let thread_share_state = share_state.clone();
       let url = url.clone().to_owned();
       let uri=Url::parse(url.as_str()).unwrap();
       
       let res = reqwest::Client::new()
                    .head(uri.as_str())
                    .send().await.unwrap();
        let header = res.headers().get(header::CONTENT_LENGTH);
        let length:u64;
        //content-range
        if None==header {
            length=match reqwest::Client::new().get(uri.as_str()).header(header::RANGE,"bytes=0-0").send().await{
                //bytes 0-0/22766251
                Ok(res)=>{
                   match res.headers().get("content-range"){
                       Some(cr)=>{
                       match cr.to_str() {
                           Ok(s)=>{
                          let lens:Vec<&str> = s.split('/').collect();
                          let ss = format!("{}",lens.get(1)
                          .unwrap().clone().to_owned());
                         u64::from_str(&ss).unwrap()
                       },
                       Err(_)=>0
                        }
                         
                    },
                    None=>0,
                    }
                    
                },
                Err(_)=>0,
            };
        }else{
            length = u64::from_str(header.unwrap().to_str().unwrap()).map_err(|_| "无效").unwrap();
        }
        let upper = length;
       let t=Task{
                        lower:0,
                        upper,
                        share_state,
                    };
        let fullfile_str = fullfile_str.clone().to_owned();
        let h= thread::spawn(move || {
            
            let mut sshare_state = thread_share_state.lock().unwrap();
            sshare_state.completed =true; 
            let down= Runtime::new().expect("Fial down").block_on(t.block_down(url.as_str(),&fullfile_str));
            match down{
           Ok(())=>{ 
               if let Some(waker) = sshare_state.waker.take() {
                    waker.wake();
                    }
                },
                Err(e)=>panic!("{}",e),
            }   
        });
       h.join().expect("Fail");
       Ok(())
    }

    async fn block_down<'a>(&self,url:&'a str,filename:&'a str)->Result<(),reqwest::Error>{
        const CHUNK_SIZE:u32 = 1024*1024*2;
        
        let fsize= self.upper;
        let mut dowloaded:u64=0;
        let pb = ProgressBar::new(fsize);
        pb.set_style(ProgressStyle::default_bar()
         .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
         .progress_chars("#>-")
         );
         //let tem_name = path.join("_temp");
         //let mut tmp_file=OpenOptions::new().create(true).append(true).open(&tem_name).unwrap();
        let mut downfile=Dowloadfile::new(filename,fsize);
        for range in blockRangeIter::BlockRangeIter::new(self.lower,fsize-1,CHUNK_SIZE).unwrap() {
                           //println!("{}",range);
                           //thread::sleep(Duration::new(0,2));
                let range:Range = range;
               let res= reqwest::Client::new().get(url)
                .header(header::RANGE,range.range_str)
                .send().await;
                match res{
                        Ok(response)=>{
                        let new_len=std::cmp::min(dowloaded+CHUNK_SIZE as u64,fsize);
                        dowloaded = new_len;
                        let status = response.status();
                        if !(status==reqwest::StatusCode::OK || status==reqwest::StatusCode::PARTIAL_CONTENT) {
                            println!("Unexpected server respone:{}",status); 

                        }
                        pb.set_position(new_len);
                        let bytes = response.bytes().await.unwrap().to_vec();
                        //let mut mem = &bytes[..];
                                //mem = unsafe{std::mem::transmute_copy(&bytes)};
                            /* match output_file.write(&bytes[..]){
                                Ok(_)=>(),
                                Err(e)=>println!("Write Error:{}",e),
                            }; */
                           match downfile.write(range.prev_size,&bytes){
                               Ok(_)=>(),
                                Err(e)=>panic!("{}",e),                           }
                           // std::io::copy(&mut mem,&mut output_file).expect("copy fial");
                           
                        },
                        Err(e)=>panic!("error {}",e),
                        }
            }
            pb.finish_with_message("downloaded");
            downfile.flush().unwrap();
            Ok(())
    }

    fn get_filename<'b>(url:&'b str,headers:&'b header::HeaderMap)->&'b str {
       let mut filename = "downfast.b";
       if !headers.is_empty() {
           filename = filename_from_headers(headers);
       }
       if filename == "" && url!="" {
           filename = filename_from_url(url);
       }

       filename
    }
}

fn new_spawner()-> (Spawner,Executor){
    const MAX_QUEUE_RUN: usize = 99_999;
    let (task_sender,ready_queue) = sync_channel(MAX_QUEUE_RUN);
    (Spawner{task_sender},Executor{ready_queue})
}

struct Executor {
    ready_queue: Receiver<Arc<Run>>,
}

impl Executor {
    fn run(&self) {
       // let mut count  = 0;
       //futures::executor::block_on(async  {
        while let Ok(run) = self.ready_queue.recv() {
            // count=count+1;
             let mut future_slot = run.future.lock().unwrap();
             if let Some(mut future) = future_slot.take() {
                 let waker = waker_ref(&run);
                 let context = &mut Context::from_waker(&*waker);
                 if let Poll::Pending = future.as_mut().poll(context) {
                     *future_slot = Some(future);
                     thread::sleep(Duration::new(0,3));
                     //println!("{} executor run the future,but is not ready.",count)
                 }else{
                     
                    println!("the run is done");
                   // break;
                 }
             }
         }
       //});
        
    }
}

#[derive(Clone)]
struct Spawner {
    task_sender:SyncSender<Arc<Run>>
}

impl Spawner {
    fn spawn(&self,future:impl Future<Output = u64>+'static +Send) {
        println!("run..");
        let future = future.boxed();
        let run = Arc::new(Run{
            future:Mutex::new(Some(future)),
            task_sender:self.task_sender.clone(),
        });
        self.task_sender.send(run).expect("too many run queued");
    }
}

struct Run{
    future: Mutex<Option<BoxFuture<'static,u64>>>,
    task_sender:SyncSender<Arc<Run>>,
}

impl ArcWake for Run{
    fn wake_by_ref(arc_self:&Arc<Self>){
        let clone = arc_self.clone();
        arc_self.task_sender.send(clone).expect("too many...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};
    use futures;

    #[test]
    fn timerfuture() {
        let (send, recv) = std::sync::mpsc::channel();
        match std::fs::File::create(".tmp"){
            Err(e)=>panic!("{}",e),
            Ok(mut f)=>{
                f.write(b"ok").unwrap();
            }
        }
        let handle = thread::spawn(move || {
            
            let p = Path::new(".tmp");
            let mut b=false;
            let result= loop {
               while let Ok(metadata)=std::fs::metadata(p){
                 b= metadata.len()>0;
                 println!("{}",metadata.len());
                 if b {
                     break;
                 }
               }
               break b;
            };
            send.send(result);
        });

        handle.join().unwrap();
        println!("list..");
        while let Ok(r) = recv.recv() {
            println!("{}",r);
        }
        println!("done.");
       // futures::executor::block_on(async{ Task::new("http://www.baidu.com").await;});
    }
}