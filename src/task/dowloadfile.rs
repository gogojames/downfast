use std::{fs::File, io::ErrorKind,fs::OpenOptions,path::PathBuf};
use std::io::{Read, Write};
use std::io::BufRead;
use std::io::BufReader;
use std::io::prelude;
#[derive(Debug)]
pub struct Dowloadfile<'a> {
    file_name:  &'a str,
    woteSize:  u64,
    file_size: u64,
}

impl<'a> Dowloadfile<'a> {
    pub fn new<'b>(file_name:&'b str,fsize:u64)-> Box<Dowloadfile> {
        let path = std::path::Path::new(".down");
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }
        Box::new(Dowloadfile {
            file_name,
            woteSize: 0,
            file_size: fsize
        })
    }

    pub fn write(&mut self,offset:u64,mut buf:&[u8])->Result<(),std::io::Error>{
       // print!("write\n");
        //let len = buf.len();
       // let file = File::open(self.file_name).unwrap();
       let tem_name = format!(".down/{}",offset);
       let path=std::path::Path::new(&tem_name);
        
        let mut tmp_file = File::create(path)?;
        std::io::copy(&mut buf,&mut tmp_file)?;
        //let mut tmp_file=OpenOptions::new().create(true).append(true).open(tem_name.as_str()).unwrap();
        /* let reader =BufReader::new(file);
        let mut line =0;
        //let mut vb=vec![0u8;self.file_size as usize];
        for b in reader.bytes(){
            let bye = b.unwrap();
            let mut vbey:Vec<u8> =Vec::new();
            if line>=offset && line< (len as u64 ) {
                if let Some(bye) = buf.get(line as usize) {
                   // println!("push {}",bye);
                   // vb.push(*bye);
                   
                   vbey.push(*bye);
                }
                
            }else{
            vbey.push(bye);
            }
            tmp_file.write(&vbey[..]).unwrap();
            
            line=line+1;
        } */
        //tmp_file.write(&vb[..]).unwrap();
        tmp_file.flush()?;
        //std::fs::remove_file(self.file_name).unwrap();
        //std::fs::rename(tem_name, self.file_name).unwrap();
        // loop {
        //     let redlen=tmp_file.read(&mut rbuf).unwrap();
        //     file.write(&rbuf[..redlen]).unwrap();
        //     if redlen< rbuf.len() {
        //         break;
        //     }
        // }
        Ok(())
    }

    pub fn flush(&mut self)->Result<(),std::io::Error>{
        let path = std::path::Path::new(".down");
        let mut tmp_file=OpenOptions::new().create(true).append(true).open(self.file_name)?;
        let mut vefname:Vec<u64> =Vec::new();
        match std::fs::read_dir(path){
            Err(why)=>panic!("{}",why),
            Ok(paths)=>for p in paths{
                let tmp = p?.path();
                if tmp.is_file() {
                    if let Some(f)=tmp.file_name() {
                     if let Some(f_str) = f.to_str(){
                         let uf:u64=f_str.parse::<u64>().unwrap();
                         vefname.push(uf);
                     }
                    }
                }
                
            }
        }
        let vefname =Dowloadfile::insert_sort(&mut vefname);

        for i in vefname.iter() {
                let t_p = format!("{}",i);
                // println!("{}",t_p);
                let mut  tmp = std::path::PathBuf::from(".down");
                tmp.push(t_p);
                if tmp.exists() {
                let mut f = File::open(&tmp.as_path())?;
                std::io::copy(&mut f, &mut tmp_file)?;
                std::fs::remove_file(tmp).unwrap();
                }
        }
          
        Ok(())
    }

    fn insert_sort(vectors: &mut Vec<u64>) -> &Vec<u64>{
        for i in 1..vectors.len(){
            let mut current = vectors[i];
            let mut j = i - 1;
            while j >= 0 && current < vectors[j]{
                let middle = vectors[j+1];
                vectors[j+1] = vectors[j];
                vectors[j] = middle;
                if j > 0{
                    /* rust 不允许while j >=0 中 j = 0 时还减 1
                    导致 j 在 while 中为负数这种危险写法*/
                    j = j - 1;  // j 递减即不断地跟左边比较
                }
            }
        }
        vectors
        
    }
}