/*
 * Copyright 2019 Balaji Jinnah and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::partition::iterator::Iterator;
use crate::partition::partition_iterator::PartitionIterator;
use crate::partition::segment_iterator::Entry;
use crate::store::batch::Batch;
use crate::store::store::Store;
use std::cell::RefCell;
use std::rc::Rc;
pub struct MergeIteartor<S> {
    partition_iterators: Vec<Rc<RefCell<PartitionIterator<S>>>>,
    current_entry: Option<Rc<Entry>>,
    current_parition: String,
    backward: bool,
}

impl<S: Store + Clone> MergeIteartor<S> {
    pub fn new(
        itrs: Vec<Rc<RefCell<PartitionIterator<S>>>>,
        backward: bool,
    ) -> Result<MergeIteartor<S>, failure::Error> {
        let mut itr = MergeIteartor {
            partition_iterators: itrs,
            current_entry: None,
            current_parition: String::from(""),
            backward: backward,
        };
        itr.next().unwrap();
        return Ok(itr);
    }
    /// entry will gove you the current entry of the iterator.
    pub fn entry(&mut self) -> Option<Rc<Entry>> {
        match self.current_entry.as_ref() {
            Some(entry) => Some(entry.clone()),
            None => None,
        }
    }

    pub fn next(&mut self) -> Result<Option<()>, failure::Error> {
        // finc the least time stamp.
        let mut prev_ts = std::u64::MAX;
        if self.backward {
            prev_ts = std::u64::MIN;
        }
        let mut inner_entry = None;
        let mut inner_iterator = None;
        // TODO: use heap instead of loop.
        for iterator in self.partition_iterators.iter_mut() {
            let entry = iterator.borrow().entry();
            if entry.is_some() {
                let entry = entry.unwrap();
                if self.backward {
                    // TODO: do it with your custom ordering
                    // Otherwise, It may be the time to do the
                    // heap thingy.
                    if entry.ts > prev_ts {
                        prev_ts = entry.ts;
                        inner_entry = Some(entry);
                        inner_iterator = Some(iterator.clone());
                    }
                    continue;
                }
                if entry.ts < prev_ts {
                    prev_ts = entry.ts;
                    inner_entry = Some(entry);
                    inner_iterator = Some(iterator.clone());
                }
            }
        }
        self.current_entry = inner_entry;
        match inner_iterator {
            Some(itr) => {
                self.current_parition = itr.borrow().partition().clone();
                itr.borrow_mut().next()?;
            }
            _ => {}
        }
        if self.current_entry.as_ref().is_none() {
            return Ok(None);
        }
        return Ok(Some(()));
    }

    pub fn partition(&self) -> &String {
        &self.current_parition
    }
}

#[cfg(test)]
mod tests {

    use crate::iterator::merge_iterator::MergeIteartor;
    use crate::partition::partition_iterator::tests::create_segment;
    use crate::partition::partition_iterator::PartitionIterator;
    use crate::partition::segment_writer::tests::{get_test_cfg, get_test_store};
    use std::cell::RefCell;
    use std::rc::Rc;
    #[test]
    fn test_merge_iterator() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        let mut itrs = Vec::new();
        let partition_name = String::from("temppartition");
        create_segment(1, partition_name.clone(), cfg.clone(), 1, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 4, store.clone());
        let partition_iterator = PartitionIterator::new(
            partition_name,
            1,
            9,
            None,
            store.clone(),
            cfg.clone(),
            false,
        )
        .unwrap()
        .unwrap();
        itrs.push(Rc::new(RefCell::new(partition_iterator)));
        let partition_name = String::from("temppartition1");
        create_segment(1, partition_name.clone(), cfg.clone(), 10, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 14, store.clone());
        let partition_iterator =
            PartitionIterator::new(partition_name, 10, 19, None, store, cfg, false)
                .unwrap()
                .unwrap();
        itrs.push(Rc::new(RefCell::new(partition_iterator)));
        let mut merge_itr = MergeIteartor::new(itrs, false).unwrap();
        let mut count = 0;
        loop {
            let ent = merge_itr.entry();
            if ent.is_none() {
                break;
            }
            count = count + 1;
            if count == 10 {
                break;
            }
            merge_itr.next();
        }
        assert!(count == 8);
    }

    #[test]
    fn test_iterator_intersection() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        let mut itrs = Vec::new();
        let partition_name = String::from("temppartition");
        create_segment(1, partition_name.clone(), cfg.clone(), 1, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 5, store.clone());
        let partition_iterator = PartitionIterator::new(
            partition_name,
            1,
            9,
            None,
            store.clone(),
            cfg.clone(),
            false,
        )
        .unwrap()
        .unwrap();
        itrs.push(Rc::new(RefCell::new(partition_iterator)));
        let partition_name = String::from("temppartition1");
        create_segment(1, partition_name.clone(), cfg.clone(), 2, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 6, store.clone());
        let partition_iterator =
            PartitionIterator::new(partition_name, 1, 10, None, store, cfg, false)
                .unwrap()
                .unwrap();
        itrs.push(Rc::new(RefCell::new(partition_iterator)));
        let mut merge_itr = MergeIteartor::new(itrs, false).unwrap();
        let mut num = 1;
        loop {
            let ent = merge_itr.entry().unwrap();
            assert_eq!(ent.ts, num);
            merge_itr.next().unwrap();
            num = num + 1;
            if num == 8 {
                break;
            }
        }
    }
    #[test]
    fn test_iterator_intersection_backward() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        let mut itrs = Vec::new();
        let partition_name = String::from("temppartition");
        create_segment(1, partition_name.clone(), cfg.clone(), 1, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 5, store.clone());
        let partition_iterator =
            PartitionIterator::new(partition_name, 1, 9, None, store.clone(), cfg.clone(), true)
                .unwrap()
                .unwrap();
        itrs.push(Rc::new(RefCell::new(partition_iterator)));
        let partition_name = String::from("temppartition1");
        create_segment(1, partition_name.clone(), cfg.clone(), 2, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 6, store.clone());
        let partition_iterator =
            PartitionIterator::new(partition_name, 1, 10, None, store, cfg, true)
                .unwrap()
                .unwrap();
        itrs.push(Rc::new(RefCell::new(partition_iterator)));
        let mut merge_itr = MergeIteartor::new(itrs, true).unwrap();
        let mut num = 8;
        loop {
            let ent = merge_itr.entry().unwrap();
            assert_eq!(ent.ts, num);
            merge_itr.next().unwrap();
            num = num - 1;
            if num == 0 {
                break;
            }
        }
    }
}
