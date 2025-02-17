## Levenshtein distance

* Optimizations
  * Distance matrix: compare directly values instead of hashes of values (10% speed up)
  * Re-using the distance matrix from previous years doesn't improve performance
  * Distance: compute graphemes in advance (30-40% speed up (overall))
  * Distance: less ifs (5% speed up)
  * Distance: use u8 (?% speed up)
  * Distance: cache dp (?% speed up)
  * Distance: iter n * n/2 (20% speed up (overall))
  * Distance: Store graphemes as u64 instead of &str (speed up 20-25% (overall))

* UTF-8 normalization:
  é can represented in two different ways
  -> normalize text before processing