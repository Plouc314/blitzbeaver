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

### Median Word

Below is a formal definition of a median word:

Let a word $w$ be formed of $c_1, c_2, \dots, c_n \in \Sigma$, an ordered sequence of elements of the alphabet $\Sigma$.

Let $W_n$ be the set of all words of length $n$:

* $W_1 = \{ c \mid c \in \Sigma \}$
* $W_n = \{ w + c \mid c \in A, w \in W_{n-1} \}$

Let $W = W_1 \cup W_2 \cup \dots \cup W_n$ the set of words of length smaller or equal to $n$.

Let $D: W \times W \to \mathbb{R}$ be a function indicating the distance between two words.

The median word of a collection of words $A = w_1, w_2, \dots, w_n$ is any word
$m \in W$ such that $\sum_{w_i \in A} D(m, w_i) = \min_{w \in W} \sum_{w_i \in A} D(w, w_i)$.

