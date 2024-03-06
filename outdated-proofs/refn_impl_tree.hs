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
append' (Leaf layout1) (Leaf layout2) = Leaf (append layout1 layout2)
append' (Leaf layout) (Branch n x y) =
  Branch (n + numNewlines layout)
         (append' (Leaf layout) x)
         (append' (Leaf layout) y)
  where
    numNewlines :: Layout -> Int
    numNewlines LErr = 0 -- doesn't matter
    numNewlines (Layout lines) = length lines - 1

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

{- Testing -}

test :: Int -> String -> Doc -> IO ()
test w s x = putStrLn ("\n" ++ s ++ ":\n" ++ pretty w x)

main = do
  test 2 "Middle" $ Text "ccc" :| Text "bb" :| Text "a"
  test 2 "Newline" $ Text "Hello" :+ Newline :+ Text "World"
  test 2 "Newline Choice" $ Newline :+ (Text "ccc" :| Text "bb")
  test 2 "Choice Newline" $ (Newline :+ Text "ccc") :| (Newline :+ Text "bb")
  test 2 "Choice with Suffix" $ (Text "cc" :| Text "d") :+ Text "x"
  test 80 "Indent" $ (2 :>> multiline) :+ (2 :>> multiline)
  test 80 "Indent" $ 2 :>> (multiline :+ multiline)
    where multiline = Text "aaaaaa" :+ Newline :+ Text "bbbbbb"
