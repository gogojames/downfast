#[test]
fn test_range(){
    get_range(0, 22766251, 1024*1204*2).unwrap();
}

 fn get_range(start:u64,end:u64,buffer_size:u32)->Result<(),&'static str>{
    //let mut sb:Vec<&str>=Vec::new();
    for range in BlockRangeIter::new(start,end,buffer_size)? {
        println!("{:?}",range); 
    }
    Ok(())
}
#[derive(Debug)]
pub struct Range {
   pub prev_size:u64,
   pub range_str:String,
}

pub struct BlockRangeIter {
    start: u64,
    end: u64,
    buffer_size:u32,
}
impl BlockRangeIter {
    pub fn new(start:u64,end:u64,buffer_size:u32) ->Result<Self,&'static str> {
        if buffer_size == 0 {
            Err("无效")?;
        }
        Ok(BlockRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for BlockRangeIter {
    type Item = Range;

    fn next(&mut self) ->Option<Self::Item> {
        if self.start > self.end {
            None
        }else{
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64,self.end - self.start+1);
            //Some(Some(format!("bytes={}-{}",prev_start,self.start-1)).unwrap())
            Some(Range{ prev_size:prev_start,range_str:format!("bytes={}-{}",prev_start,self.start-1)})
        }
    }
}