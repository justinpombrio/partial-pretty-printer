import Data.List (intercalate)

data Doc
  = Err
  | Empty
  | Text String
  | Newline
  | Indent Int Doc
  | Flat Doc
  | Concat Doc Doc
  | Choice Doc Doc

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
pp w (Indent i x) n k = pp w x n (k . indent i)
pp w (Flat x) n k = pp w x n (k . flatten)
pp w (Choice x y) n k = pick w n (pp w x n k) (pp w y n k)
pp w (Concat x y) n k
  = pp w x n (\x -> pp w y (n + numNewlines x) (\y -> k (append x y)))

{- Testing -}

infixr 6 <+>
infixr 5 <|>
(<+>) = Concat
(<|>) = Choice

display :: Layout -> String
display LErr = "!"
display (Layout lines) = intercalate "\n" lines

pretty :: Int -> Doc -> String
pretty w x = display (pp w x 0 id)

test :: Int -> String -> Doc -> IO ()
test w s x = putStrLn ("\n" ++ s ++ ":\n" ++ pretty w x)

main = do
  test 2 "Middle" $ Text "ccc" <|> Text "bb" <|> Text "a"
  test 2 "Newline" $ Text "Hello" <+> Newline <+> Text "World"
  test 2 "Newline Choice" $ Newline <+> (Text "ccc" <|> Text "bb")
  test 2 "Choice Newline" $ (Newline <+> Text "ccc") <|> (Newline <+> Text "bb")
  test 2 "Choice with Suffix" $ (Text "cc" <|> Text "d") <+> Text "x"
