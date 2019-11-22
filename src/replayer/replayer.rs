use crate::config::config::Config;
use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, read_dir, File};
use std::collections::{HashSet, HashMap};
use failure::Error;

/// Replayer will help us to build the index file and posting list, if index file is not 
/// persisted due to crash.
pub struct Replayer {
    cfg: Config,
}

impl Replayer{
    /// replay function replay the crashed segment files.
    pub fn replay(&self) -> Result<(), Error>{
        let partition_path = Path::new(&self.cfg.dir).join("partition");
        create_dir_all(&partition_path)?;

        // iterate over every partition and find the crashed segment
        // files. 
        for partition in read_dir(&partition_path)?{
            let partition = partition?;
            if partition.path().is_dir(){

            }
        }
        Ok(())    
    }
    
    /// replay_segments will check whether segment file have 
    pub fn replay_segments(&self, partition_path: PathBuf) -> Result<(), Error>{
        let mut fsa_files = HashSet::new();
        let mut segment_files = Vec::new();

        for entry in read_dir(&partition_path)?{
            let entry = entry?;
            let entry_path = entry.path();
            assert_eq!(entry_path.is_file(),true);
            
            // check whether it is fsa or segment file.
            let extension = partition_path.extension().unwrap();
            if extension == "fsa" {
                let file_name = partition_path.file_stem().unwrap();
                let splits = file_name.to_str().unwrap().split("_");
                let splits = splits.collect::<Vec<&str>>();
                fsa_files.insert(splits[2].to_string());
                continue;
            }
            assert_eq!(extension, "segment");
            let file_name = partition_path.file_stem().unwrap();
            segment_files.push(file_name.to_str().unwrap().parse::<u64>().unwrap());
        }
        // reverse it because only last file doesn't have partition.
        segment_files.reverse();
        for segment_file in segment_files{
            if !fsa_files.contains(&segment_file.to_string()){
                // this segment file doesn't have index file, so
                // rebuild the index file. 
            }
        }
        Ok(())
    }

    pub fn build_fst(&self, partition_path: &PathBuf, segment_id: u64) -> Result<(), Error>{
        let mut file = File::open(partition_path.join(format!("{}.segment",segment_id)))?;
        Ok(())
    }
}



