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
    
    display :: Layout -> String
    display LErr = "Error!"
    display (Layout lines) = intercalate "\n" lines
    
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
    
    pick :: Int -> Int -> Layout -> Layout -> Layout
    pick w _ LErr LErr = LErr
    pick w _ (Layout lines) LErr = Layout lines
    pick w _ LErr (Layout lines) = Layout lines
    pick w n (Layout lines1) (Layout lines2) =
      if length (lines1 !! n) <= w
      then Layout lines1
      else Layout lines2
    
    numNewlines :: Layout -> Int
    numNewlines LErr = 0 -- doesn't matter
    numNewlines (Layout lines) = length lines - 1
    
    data Layouts = Branch Int Layouts Layouts
                 | Leaf Layout
    
    flatten' :: Layouts -> Layouts
    flatten' (Branch n x y) = Branch n (flatten' x) (flatten' y)
    flatten' (Leaf layout) = Leaf (flatten layout)
    
    indent' :: Int -> Layouts -> Layouts
    indent' i (Branch n x y) = Branch n (indent' i x) (indent' i y)
    indent' i (Leaf layout) = Leaf (indent i layout)
    
    append' :: Layouts -> Layouts -> Layouts
    append' (Branch n x y) z = Branch n (append' x z) (append' y z)
    append' (Leaf layout) (Branch n x y) =
      Branch (n + numNewlines layout)
             (append' (Leaf layout) x)
             (append' (Leaf layout) y)
    append' (Leaf layout1) (Leaf layout2) = Leaf (append layout1 layout2)
    
    resolve :: Int -> Layouts -> Layout
    resolve _ (Leaf layout) = layout
    resolve w (Branch n x y) =
      pick w n (resolve w x) (resolve w y)
    
    pp :: Doc -> Layouts
    pp Err = Leaf LErr
    pp Empty = Leaf (Layout [""])
    pp (Text t) = Leaf (Layout [t])
    pp Newline = Leaf (Layout ["", ""])
    pp (Flat x) = flatten' (pp x)
    pp (i :>> x) = indent' i (pp x)
    pp (x :+ y) = append' (pp x) (pp y)
    pp (x :| y) = Branch 0 (pp x) (pp y)
    
    pretty :: Int -> Doc -> String
    pretty w x = display (resolve w (pp x))

## Proofs of Laws

#### Shorthands

    ε = Empty
    ! = Error
    ⬐ = Newline
    "" = (Text "")
    "t" = Text t
    i >> x = i :>> x
    x + y = x :+ y
    x | y = x :| y
    ' '*i = Text (replicate i ' ')
            or just `replicate i ' '` depending on context
    Lay = Leaf . Layout

I typically omit the width argument in pp, pick, etc., since it never changes.

`inc n x` means "increment the line number `n` to include any newlines from `x`". So
`inc n x = n + numNewlines x`

#### Auxilliary laws

I verified these mentally or on paper. A little tedius, but straightforward.

    append (Layout [""]) y = y
    append x (Layout [""]) = x
    append (append x y) z = append x (append y z)
    append' (append' x y) z = append' x (append' y z)

    append (Layout [t1]) (Layout [t2]) = Layout [t1 ++ t2]
    append LErr _ = LErr
    append _ LErr = LErr

    indent i (Layout [t]) = Layout [t]
    indent i (Layout ["", ""]) = Layout ["", replicate i ' ']
    indent i (append x y) = append (indent i x) (indent i y)
    indent' i (append' x y) = append' (indent' i x) (indent' i y)
    indent i LErr = LErr
    indent i (indent j x) = indent (i + j) x
    indent 0 x = x

    flatten (Layout [t]) = Layout [t]
    flatten (Layout ["", ""]) = LErr
    flatten (append x y) = append (flatten x) (flatten y)
    flatten (flatten x) = flatten x
    flatten LErr = LErr

#### Concatenation Laws

Concat-unit:

      pp (ε + y)
    = append' (pp ε) (pp y)     -- Concat
    = append' (Lay [""]) (pp y) -- Empty
    = pp y                      -- law of append

      pp (x + ε)
    = append' (pp x) (pp ε)     -- Concat
    = append' (pp x) (Lay [""]) -- Empty
    = pp x                      -- law of append

Concat-assoc:

      pp ((x + y) + z)
    = append' (pp (x + y)) (pp z)            -- Concat
    = append' (append' (pp x) (pp y)) (pp z) -- Concat
    = append' (pp x) (append' (pp y) (pp z)) -- law of append'

      pp (x + (y + z))
    = append' (pp x) (pp (y + z))            -- Concat
    = append' (pp x) (append' (pp y) (pp z)) -- Concat

#### Text Laws

Text-empty:

      pp ""
    = Lay [""] -- Text
    = pp ε     -- Empty, in reverse

Text-append:

      pp ("t1" + "t2")
    = append' (pp "t1") (pp "t2")   -- Concat
    = append' (Lay [t1]) (Lay [t2]) -- Text
    = Lay [t1 ++ t2]                -- law of append
    = pp "t1 ++ t2"                 -- Text, in reverse

#### Indentation Laws

Indent-absorb-empty:

      pp (i >> ε)
    = indent' i (pp ε)     -- Indent
    = indent' i (Lay [""]) -- Empty
    = Lay [""]             -- law of indent

Indent-absorb-text:

      pp (i >> "t")
    = indent' i (pp "t")   -- Indent
    = indent' i (Lay [t])  -- Text
    = Lay [t]              -- law of indent
    = pp "t"               -- Text, in reverse

Indent-newline:

      pp (i >> ⬐)
    = indent' (pp ⬐)            -- Indent
    = indent' (Lay ["", ""])    -- Newline
    = Lay ["", replicate i ' '] -- law of indent

      pp (⬐ + replicate i ' ')
    = append' (pp ⬐) (pp (replicate i ' '))          -- Concat
    = append' (Lay ["", ""]) (Lay [replicate i ' ']) -- Text, Newline
    = Lay ["", replicate i ' ']                      -- (simplify)

Indent-distr-append:

      pp (i >> (x + y))
    = indent' i (pp (x + y))                        -- Indent
    = indent' i (append' (pp x) (pp y))             -- Concat
    = append' (indent' i (pp x)) (indent' i (pp y)) -- law of indent

      pp ((i >> x) + (i >> y))
    = append' (pp (i >> x)) (pp (i >> y))           -- Concat
    = append' (indent' i (pp x)) (idnent' i (pp y)) -- Indent

Indent-distr-choice:

      pp (i >> (x | y))
    = indent' i (pp (x | y))                     -- Indent
    = indent' i (Br 0 (pp x) (pp y))             -- Choice
    = Br 0 (indent' i (pp x)) (indent' i (pp y)) -- (simplify)

      pp ((i >> x) | (i >> y))
    = Br 0 (pp (i >> x)) (pp (i >> y))           -- Choice
    = Br 0 (indent' i (pp x)) (indent' i (pp y)) -- Indent

Indent-identity:

      pp (0 >> x)
    = indent' 0 (pp x) -- Indent
    = pp x             -- law of indent

Indent-compose:

      pp (i >> (j >> x))
    = indent' i (indent' j (pp x)) -- Indent
    = indent' (i + j) (pp x)       -- law of indent

#### Flattening Laws

Flat-absorb-empty:

      pp (Flat ε)
    = flatten' (pp ε)     -- Flat
    = flatten' (Lay [""]) -- Empty
    = Lay [""]            -- law of flatten
    = pp ε                -- Empty, in reverse

Flat-absorb-text:

      pp (Flat "t")
    = flatten' (pp "t")  -- Flat
    = flatten' (Lay [t]) -- Text
    = Lay [t]            -- law of flatten
    = pp "t"             -- Text, in reverse

Flat-newline:

      pp (Flat ⬐)
    = flatten' (pp ⬐)         -- Flat
    = flatten' (Lay ["", ""]) -- Newline
    = Leaf LErr               -- law of flatten
    = pp Err                  -- Error, in reverse

Flat-distr-append:

      pp (Flat (x + y))
    = flatten' (pp (x + y))                       -- Flat
    = flatten' (append' (pp x) (pp y))            -- Concat
    = append' (flatten' (pp x)) (flatten' (pp y)) -- law of flatten

      pp ((Flat x) + (Flat y))
    = append' (pp (Flat x)) (pp (Flat y))         -- Concat
    = append' (flatten' (pp x)) (flatten' (pp y)) -- flatten

Flat-distr-choice:

      pp (Flat (x | y))
    = flatten' (pp (x | y))                    -- Flat
    = flatten' (Br 0 (pp x) (pp y))            -- Choice
    = Br 0 (flatten' (pp x)) (flatten' (pp y)) -- (simplify)

      pp ((Flat x) | (Flat y))
    = Br 0 (pp (Flat x)) (pp (Flat y))         -- Choice
    = Br 0 (flatten' (pp x)) (flatten' (pp y)) -- Flat

Flatten-compose:

      pp (Flat (Flat x))
    = flatten' (flatten' (pp x)) -- Flat
    = flatten' (pp x)            -- law of flatten
    = pp (Flat x)                -- Flat, in reverse

#### Error Laws

Error-append:

      pp (! + y)
    = append' (pp !) (pp y)      -- Concat
    = append' (Leaf LErr) (pp y) -- Error
    = Leaf LErr                  -- law of append
    = pp !                       -- Error, in reverse

      pp (x + !)
    = append' (pp x) (pp !)      -- Concat
    = append' (pp x) (Leaf LErr) -- Error
    = Leaf LErr                  -- law of append
    = pp !                       -- Error, in reverse

Error-indent:

      pp (i >> !)
    = indent' i (pp !)      -- Indent
    = indent' i (Leaf LErr) -- Error
    = Leaf LErr             -- law of indent
    = pp !                  -- Error, in reverse

Error-flat:

      pp (Flat !)
    = flatten' (pp !)      -- Flat
    = flatten' (Leaf LErr) -- Error
    = Leaf LErr            -- law of flatten
    = pp !                 -- Error, in reverse

Error-choice:

      pp (! | y)
    = Br 0 (pp !) (pp y)      -- Choice
    = Br 0 (Leaf LErr) (pp y) -- Error
    = pp y                    -- (see below)

      pp (x | !)
    = Br 0 (pp x) (pp !)      -- Choice
    = Br 0 (pp x) (Leaf LErr) -- Error
    = pp x                    -- (see below)

(`Br n (Leaf LErr) y` and `Br n x (Leaf LErr)` are contextually equivalent in calls to `pp` and
`resolve` because (i) `pp` preserves them, and (ii) `resolve` calls `pick` on them, which always
chooses the non-error branch.)

#### Choice Laws

Choice-assoc:

      pp ((x | y) | z)
    = Br 0 (pp (x | y)) (pp z)         -- Choice
    = Br 0 (Br 0 (pp x) (pp y)) (pp z) -- Choice
    = Br 0 (pp x) (Br 0 (pp y) (pp z)) -- (see below)

      pp (x | (y | z))
    = Br 0 (pp x) (pp (y | z))         -- Choice
    = Br 0 (pp x) (Br 0 (pp y) (pp z)) -- Choice

(`Br m x (Br n y z)` is contextually equivalent to `Br m (Br n x y) z` in calls to `pp` and
`resolve` because (i) `pp` preserves them, and (ii) resolve folds over them using `pick`, which is
associative.)

Choice-distr-text-left:

      pp ("t" + (x | y))
    = append' (pp "t") (pp (x | y))                              -- Concat
    = append' (Lay [t]) (Br 0 (pp x) (pp y))                     -- Text, Choice
    = Br 0 (append' (Lay [t]) (pp x)) (append' (Lay [t]) (pp y)) -- (simplify)

      pp (("t" + x) | ("t" + y))
    = Br 0 (pp ("t" + x)) (pp ("t" + y))                         -- Choice
    = Br 0 (append' (pp "t") (pp x)) (append' (pp "t") (pp y))   -- Concat
    = Br 0 (append' (Lay [t]) (pp x)) (append' (Lay [t]) (pp y)) -- Text

Choice-distr-right:

      pp ((x | y) + z)
    = append' (pp (x | y)) (pp z)                                -- Concat
    = append' (Br 0 (pp x) (pp y)) (pp z)                        -- Choice
    = Br 0 (append' (pp x) (pp z)) (append' (pp y) (pp z))       -- (simplify)

      pp ((x + z) | (y + z))
    = Br 0 (pp (x + z)) (pp (y + z))                             -- Choice
    = Br 0 (append' (pp x) (pp z)) (append' (pp y) (pp z))       -- Concat
