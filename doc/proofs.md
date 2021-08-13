# Proofs

## Reference Implementation

    import Data.List (intercalate)
    
    data Doc
      = Err
      | Empty
      | Text String
      | Newline
      | Int :>> Doc
      | Flat Doc
      | Doc :+ Doc
      | Doc :| Doc
    
    
    data Layout = Layout [String]
                | LErr
    
    flatten :: Layout -> Layout
    flatten LErr = LErr
    flatten (Layout lines) =
      if length lines == 1
      then Layout lines
      else LErr
    
    indent :: Int -> Layout -> Layout
    indent _ LErr = LErr
    indent i (Layout (line : lines)) =
      Layout (line : map addSpaces lines)
      where addSpaces line = replicate i ' ' ++ line
    
    append :: Layout -> Layout -> Layout
    append (Layout lines1) (Layout lines2) =
      Layout (init lines1 ++ [middleLine] ++ tail lines2)
      where middleLine = (last lines1 ++ head lines2)
    append _ _ = LErr
    
    numNewlines :: Layout -> Int
    numNewlines LErr = 0 -- doesn't matter
    numNewlines (Layout lines) = length lines - 1
    
    pick :: Int -> Int -> Layout -> Layout -> Layout
    pick w _ LErr LErr = LErr
    pick w _ (Layout lines) LErr = Layout lines
    pick w _ LErr (Layout lines) = Layout lines
    pick w n (Layout lines1) (Layout lines2) =
      if length (lines1 !! n) <= w
      then Layout lines1
      else Layout lines2
    
    pp :: Int -> Doc -> Int -> (Layout -> Layout) -> Layout
    pp w Err _ k = k LErr -- should equal LErr
    pp w Empty _ k = k (Layout [""])
    pp w (Text t) _ k = k (Layout [t])
    pp w Newline _ k = k (Layout ["", ""])
    pp w (i :>> x) n k = pp w x n (k . indent i)
    pp w (Flat x) n k = pp w x n (k . flatten)
    pp w (x :| y) n k = pick w n (pp w x n k) (pp w y n k)
    pp w (x :+ y) n k = pp w x n (\x ->
      pp w y (n + numNewlines x) (\y -> k (append x y)))
    
    pretty :: Int -> Doc -> String
    pretty w x = display (pp w x 0 id)
      where
        display :: Layout -> String
        display LErr = "!"
        display (Layout lines) = intercalate "\n" lines

## Proofs of Laws

#### Shorthands

    ε = Empty
    ! = Error
    ⬐ = Newline
    "" = (Text "")
    "t" = Text t
    i >> x = Indent i x
    x + y = x :+ y
    x | y = x :| y
    ' '*i = Text (replicate i ' ')
            or just `replicate i ' '` depending on context

I typically omit the width argument in pp, pick, etc., since it never changes.

`inc n x` means "increment the line number `n` to include any newlines from `x`". So
`inc n x = n + numNewlines x`

#### Auxilliary laws

I verified these mentally or on paper. A little tedius, but straightforward.

    append (Layout [""]) y = y
    append x (Layout [""]) = x
    append (append x y) z = append x (append y z)
    append (Layout [t1]) (Layout [t2]) = Layout [t1 ++ t2]
    append LErr _ = LErr
    append _ LErr = LErr

    inc n (append x y) = inc (inc n x) y

    pick w (pick w x y) z = pick w x (pick w y z)
    pick w x x = x
    pick w LErr y = y
    pick w x LErr = x

    indent i (Layout [t]) = Layout [t]
    indent i (append x y) = append (indent i x) (indent i y)
    indent i LErr = LErr

    flatten (Layout [t]) = Layout [t]
    flatten (Layout ["", ""]) = LErr
    flatten (append x y) = append (flatten x) (flatten y)
    flatten LErr = LErr

Also, for every continuation `k` that is ever passed to `pp`, `k LErr = LErr`. You can see
this because `k`s are constructed only by the indent, flat, and append cases, which call only
`indent`, `flatten`, `append`, and `pp`, all of which produce `LErr` when passed a `LErr`.

#### Concatenation Laws

Concat-unit:

      pp (ε + y) n k
    = pp ε n (\x -> pp y (inc n x) (\y -> k (append x y)))        -- Concat
    = (\x -> pp y (inc n x) (\y -> k (append x y))) (Layout [""]) -- Empty
    = pp y n (\y -> k (append (Layout [""]) y))                   -- (simplify)
    = pp y n (\y -> k y)                                          -- law of append
    = pp y n k                                                    -- (simplify)

      pp (x + ε) n k
    = pp x n (\x -> pp ε (inc n x) (\y -> k (append x y))) -- Concat
    = pp x n (\x -> (\y -> k (append x y)) (Layout [""]))  -- Empty
    = pp x n (\x -> k (append x (Layout [""])))            -- (simplify)
    = pp x n (\x -> k x)                                   -- law of append
    = pp x n k                                             -- (simplify)

Concat-assoc:

      pp ((x + y) + z) n k
    = pp (x + y) n (\xy -> pp z (inc n xy) (\z -> k (append xy z)))     -- Concat
    = pp x n (\x -> pp y (inc n x) (\y ->
        (\xy -> pp z (inc n xy) (\z -> k (append xy z))) (append x y))) -- Concat
    = pp x n (\x -> pp y (inc n x) (\y ->
        pp z (inc n (append x y)) (\z -> k (append (append x y) z))))   -- (simplify)
    = pp x n (\x -> pp y (inc n x) (\y ->
        pp z (inc (inc n x) y) (\z -> k (append x (append y z)))))      -- law of append, inc

      pp (x + (y + z)) n k
    = pp x n (\x -> pp (y + z) (inc n x) (\yz -> k (append x yz)))     -- Concat
    = pp x n (\x -> pp y (inc n x) (\y ->
        pp z (inc (inc n x) y) (\z ->
          (\yz -> k (append x yz)) (append y z))))                     -- Concat
    = pp x n (\x -> pp y (inc n x) (\y ->
        pp z (inc (inc n x) y) (\z -> k (append x (append y z)))))     -- (simplify)

#### Text Laws

Text-empty:

      pp "" n k
    = k (Layout "")  -- Text
    = pp ε n k       -- Empty (in reverse)

Text-append:

      pp ("t1" + "t2") n k
    = pp "t1" n (\x -> pp "t2" (inc n x) (\y -> k (append x y)))     -- Concat
    = (\x -> pp "t2" (inc n x) (\y -> k (append x y))) (Layout [t1]) -- Text
    = (\x -> (\y -> k (append x y)) (Layout [t2]) (Layout [t1])      -- Text
    = k (append (Layout [t1]) (Layout [t2]))                         -- (simplify)
    = k (Layout [t1 ++ t2])                                          -- law of append
    = pp "t1 ++ t2" n k                                              -- Text

#### Indentation Laws

Indent-absorb-empty:

      pp (i >> ε) n k
    = pp ε n (k . indent i)        -- Indent
    = (k . indent i) (Layout [""]) -- Empty
    = k (Layout [""])              -- law of indent
    = pp ε n k                     -- Empty, in reverse

Indent-absorb-text:

      pp (i >> "t") n k
    = pp "t" n (k . indent i)     -- Indent
    = (k . indent i) (Layout [t]) -- Text
    = k (Layout [t])              -- law of indent
    = pp "t" n k                  -- Text, in reverse

Indent-newline:

      pp (i >> ⬐) n k
    = pp ⬐ n (k . indent i)            -- Indent
    = (k . indent i) (Layout ["", ""]) -- Newline
    = k (Layout ["", replicate i ' '])

      pp (⬐ + ' '*i) n k
    = pp ⬐ n (\x -> pp ' '*i) (inc n x) (\y -> k (append x y))            -- Concat
    = (\x -> pp ' '*i (inc n x) (\y -> k (append x y))) (Layout ["", ""]) -- Newline
    = pp ' '*i (n + 1) (\y -> k (append (Layout ["", ""]) y))             -- (simplify)
    = (\y -> k (append (Layout ["", ""]) y)) (Layout [' '*i])             -- Text
    = k (append (Layout ["", ""]) (Layout [' '*i]))                       -- (simplify)
    = k (Layout ["", ' '*i])                                              -- (simplify)

Indent-distr-append:

      pp (i >> (x + y)) n k
    = pp (x + y) n (k . indent i)                                                    -- Indent
    = pp x n (\x -> pp y (inc n x) (\y -> (k . indent i) (append x y)))              -- Concat
    = pp x n (\x -> pp y (inc n x) (\y -> k (indent i (append x y))))                -- (simplify)

      pp ((i >> x) + (i >> y)) n k
    = pp (i >> x) n (\x -> pp (i >> y) (inc n x) (\y -> k (append x y)))             -- Concat
    = pp x n ((\x -> pp (i >> y) (inc n x) (\y -> k (append x y))) . indent i)       -- Indent
    = pp x n ((\x -> pp y (inc n x) ((\y -> k (append x y)) . indent i)) . indent i) -- Indent
    = pp x n (\x -> pp y (inc n x) (\y -> k (append (indent i x) (indent i y))))     -- (simplify)
    = pp x n (\x -> pp y (inc n x) (\y -> k (indent i (append x y))))                -- law of indent

Indent-distr-choice:

      pp (i >> (x | y)) n k
    = pp (x | y) n (k . indent i)                            -- Indent
    = pick n (pp x n (k . indent i)) (pp y n (k . indent i)) -- Choice

      pp ((i >> x) | (i >> y)) n k
    = pick n (pp (i >> x) n k) (pp (i >> y) n k)             -- Choice
    = pick n (pp x n (k . indent i)) (pp y n (k . indent i)) -- Indent

#### Flattening Laws

Flat-absorb-empty:

      pp (Flat ε) n k
    = pp ε n (k . flatten)        -- Flat
    = (k . flatten) (Layout [""]) -- Empty
    = k (Layout [""])             -- law of flatten
    = pp ε n k                    -- Empty, in reverse

Flat-absorb-text:

      pp (Flat "t") n k
    = pp "t" n (k . flatten)     -- Flat
    = (k . flatten) (Layout [t]) -- Text
    = k (Layout [t])             -- law of flatten
    = pp "t" n k                 -- Text, in revese

Flat-newline:

      pp (Flat ⬐) n k
    = pp ⬐ n (k . flatten)            -- Flat
    = (k . flatten) (Layout ["", ""]) -- Newline
    = k LErr                          -- law of flatten
    = pp ! n k                        -- Error, in reverse

Flat-distr-append:

      pp (Flat (x + y)) n k
    = pp (x + y) n (k . flatten)                                                   -- Flat
    = pp x n (\x -> pp y (inc n x) (\y -> (k . flatten) (append x y)))             -- Concat
    = pp x n (\x -> pp y (inc n x) (\y -> k (append (flatten x) (flatten y))))     -- law of flatten

      pp ((Flat x) + (Flat y)) n k
    = pp (Flat x) n (\x -> pp (Flat y) (inc n x) (\y -> k (append x y)))           -- Concat
    = pp x n ((\x -> pp (Flat y) (inc n x) (\y -> k (append x y))) . flatten)      -- Flat
    = pp x n (\x -> pp (Flat y) n (\y -> k (append (flatten x) y)))                -- (simplify)
    = pp x n (\x -> pp y n ((\y -> k (append (flatten x) y)) . flatten))           -- Flat
    = pp x n (\x -> pp y n (\y -> k (append (flatten x) (flatten y))))             -- (simplify)

(The line numbers are different here. But either `x` is multi-line, in which case both expressions
error, or `x` is single-line, in which case the line numbers match.)

Flat-distr-choice:

      pp (Flat (x | y)) n k
    = pp (x | y) n (k . flatten)                         -- Flat
    = pick (pp x n (k . flatten)) (pp y n (k . flatten)) -- Choice
    
      pp ((Flat x) | (Flat y)) n k
    = pick (pp (Flat x) n k) (pp (Flat y) n k)           -- Choice
    = pick (pp x n (k . flatten)) (pp y n (k . flatten)) -- Flat

#### Error Laws

Error-append:

      pp (! + y) n k
    = pp ! n (\x -> pp y (inc n x) (\y -> k (append x y))) -- Concat
    = (\x -> pp y (inc n x) (\y -> k (append x y))) LErr   -- Error
    = pp y n (\y -> k (append LErr y))                     -- (simplify)
    = pp y n (\y -> k LErr)                                -- law of append
    = k LErr                                               -- by below lemma
    = pp ! n k                                             -- Error, in reverse

      pp (x + !) n k
    = pp x n (\x -> pp ! (inc n x) (\y -> k (append x y))) -- Concat
    = pp x n (\x -> (\y -> k (append x y)) LErr)           -- Error
    = pp x n (\x -> k (append x LErr))                     -- (simplify)
    = pp x n (\x -> k LErr)                                -- (simplify)
    = k LErr                                               -- by below lemma
    = pp ! n k

    or just:
      pp (! + y) n k = (expand append) = LErr = pp ! n k
      pp (x + !) n k = (expand append) = pp x n (\x -> LErr)  =(below lemma) pp ! n k
    if we use the simpler definition of !

Error-indent:

      pp (i >> !) n k
    = pp ! n (k . indent i) -- Indent
    = (k . indent i) LErr   -- Error
    = k LErr                -- law of indent
    = pp ! n k              -- Error, in reverse

Error-flat:

      pp (flat !) n k
    = pp ! n (k . flatten) -- Flat
    = (k . flatten) LErr   -- Error
    = k LErr               -- law of flatten
    = pp ! n k             -- Error, in reverse

Error-choice:

    pp (! | y) n k
  = pick n (pp ! n k) (pp y n k) -- Choice
  = pick n (k LErr) (pp y n k)   -- Error
  = pick n LErr (pp y n k)       -- property of k
  = pp y n k                     -- law of pick

    pp (x | !) n k
  = pick n (pp x n k) (pp ! n k) -- Choice
  = pick n (pp x n k) (k LErr)   -- Error
  = pick n (pp x n k) LErr       -- property of k
  = pp x n k                     -- law of pick

Mini lemma:

    The lemma:
      pp x n (\_ -> k LErr) = k LErr

    Proof. Induct on x:
      pp Err n (\_ -> k LErr) = k LErr
      pp ε n (\_ -> k LErr) = (\_ -> k LErr) (Layout [""]) = k LErr
      pp "t" n (\_ -> k LErr) = (\_ -> k LErr) (Layout [""]) = k LErr
      pp ⬐ n (\_ -> k LErr)
        = (\_ -> k LErr) (Layout ["", ""])                      -- Newline
        = k LErr                                                -- (simplify)
      pp (i >> x) n (\_ -> k LErr)
        = pp x n ((\_ -> k LErr) . indent i)                    -- Indent
        = pp x n (\_ -> k LErr)                                 -- (simplify)
        = k LErr                                                -- inductive hypothesis
      pp (Flatten x) n (\_  -> k LErr)
        = pp x n ((\_ -> k LErr) . flatten)                     -- Flat
        = pp x n (\_ -> k LErr)                                 -- (simplify)
        = k LErr                                                -- inductive hypothesis
      pp (x + y) n (\_ -> k LErr)
        = pp x n (\x -> pp y (inc n x) (\y -> (\_ -> k LErr) (append x y)))
        = pp x n (\x -> pp y (inc n x) (\y -> k LErr))          -- (simplify)
        = pp x n (\x -> k LErr)                                 -- inductive hypothesis
        = k LErr                                                -- inductive hypothesis
      pp (x | y) n (\_ -> k LErr)
        = pick (pp x n (\_ -> k LErr)) (pp y n (\_ -> k LErr))  -- Choice
        = pick (k LErr) (k LErr)                                -- inductive hypothesis
        = k LErr                                                -- law of pick

#### Choice Laws

Choice-assoc:

      pp ((x | y) | z) n k
    = pick (pp (x | y) n k) (pp z n k)             -- Choice
    = pick (pick (pp x n k) (pp y n k)) (pp z n k) -- Choice
    = pick (pp x n k) (pick (pp y n k) (pp z n k)) -- law of pick
  
      pp (x | (y | z)) n k
    = pick (pp x n k) (pp (y | z) n k)             -- Choice
    = pick (pp x n k) (pick (pp y n k) (pp z n k)) -- Choice

Choice-distr-text-left:

      pp ("t" + (y |z)) n k
    = pp "t" n (\x -> pp (y | z) (inc n x) (\yz -> k (append x yz)))     -- Concat
    = (\x -> pp (y | z) (inc n x) (\yz -> k (append x yz))) (Layout [t]) -- Text
    = pp (y | z) n (\yz -> k (append (Layout [t]) yz))                   -- (simplify)
    = pick (pp y n (\yz -> k (append (Layout [t]) yz)))
           (pp z n (\yz -> k (append (Layout [t]) yz)))                  -- Choice
    = pick (pp ("t" + y) n k) (pp ("t" + z) n k)                         -- by below lemma
    = pp (("t" + y) | ("t" + z)) n                                       -- Choice, in reverse
  
Mini lemma:

      pp ("t" + y) n k
    = pp "t" n (\x -> pp y (inc n x) (\y -> k (append x y)))     -- Concat
    = (\x -> pp y (inc n x) (\y -> k (append x y))) (Layout [t]) -- Text
    = pp y n (\y -> k (append (Layout [t]) y))                   -- (simplify)

Choice-distr-right:

      pp ((x | y) + z) n k
    = pp (x | y) n (\xy -> pp z (inc n xy) (\z -> k (append xy z)))  -- Concat
    = pick (pp x n (\xy -> pp z (inc n xy) (\z -> k (append xy z)))
           (pp y n (\xy -> pp z (inc n xy) (\z -> k (append xy z)))) -- Choice

      pp ((x + z) | (y + z)) n k
    = pick (pp n (x + z) k) (pp n (y + z) k)                      -- Choice
    = pick (pp x n (\x -> pp z (inc n x) (\z -> k (append x z))))
           (pp y n (\y -> pp z (inc n y) (\z -> k (append y z)))) -- Concat

