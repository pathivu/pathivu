## Goals

- should ingest large amount of data.
- easy to query (so indexing has to be done)


## Storage Format

- Each app will be seperate logically as partition. Each partition will have segment files in a configurable size.
  Each segment file will have fst assocaiated with it. So, that user can search using fst. fst will hold the offset
  of the log line. If we develoed the query language, we can make posting list to do the intersection. (Further work
  , if the logs come with warn error then the warn will have all the integer. So building posting list will consume
  all the memory. one thing is spliting posting list but have to find how about doing the intersection stuff)
  And also segement files will be flushed to the disk based on the size of fst. If the size of fst is big, we'll 
  flush directly. 

- Need to serach will the fst gives us vec to store as result. Otherwise, we may have to rely on rocksdb to store the
  the posting list. (Lemme check tanitvy. Yep cheating always. This guys stores posting list in a file we don't need
  that kind of complexity. rocksdb or sled will do the job :)).

- Each segement file will have starting and ending date. and do a binary serach to pick the segment files and execute
  the query. But the problem is the latest is stored at the tails of the log. while you showing back to the user, it
  should be always back to front.

- magic number|line|line| date index.

## Components

### segement registry.



### TODO CASES.

- how to handle inmemory segments.
- if there is no segment for a parition.

  