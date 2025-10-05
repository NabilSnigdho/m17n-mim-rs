# Input method Specification

## Structure of an input method file

An input method is defined in a \*.mim file with this format.

```
(input-method LANG NAME)

(description (_ "DESCRIPTION"))

(title "TITLE-STRING")

(map
  (MAP-NAME
    (KEYSEQ MAP-ACTION MAP-ACTION ...)        <- rule
    (KEYSEQ MAP-ACTION MAP-ACTION ...)        <- rule
    ...)
  (MAP-NAME
    (KEYSEQ MAP-ACTION MAP-ACTION ...)        <- rule
    (KEYSEQ MAP-ACTION MAP-ACTION ...)        <- rule
    ...)
  ...)

(state
  (STATE-NAME
    (MAP-NAME BRANCH-ACTION BRANCH-ACTION ...)   <- branch
    ...)
  (STATE-NAME
    (MAP-NAME BRANCH-ACTION BRANCH-ACTION ...)   <- branch
    ...)
  ...)
```

Lowercase letters and parentheses are literals, so they must be written as they are. Uppercase letters represent arbitrary strings.

KEYSEQ specifies a sequence of keys in this format:

```
  (SYMBOLIC-KEY SYMBOLIC-KEY ...)
```

where SYMBOLIC-KEY is the keysym value returned by the xev command. For instance

```
  (n i)
```

represents a key sequence of <n> and <i>. If all SYMBOLIC-KEYs are ASCII characters, you can use the short form

```
  "ni"
```

instead. Consult [Input Method](m17nDBFormat.html#mdbIM) for Non-ASCII characters.

Both MAP-ACTION and BRANCH-ACTION are a sequence of actions of this format:

```
  (ACTION ARG ARG ...)
```

The most common action is `insert`, which is written as this:

```
  (insert "TEXT")
```

But as it is very frequently used, you can use the short form

```
  "TEXT"
```

If `"TEXT"` contains only one character "C", you can write it as

```
  (insert ?C)
```

or even shorter as

```
  ?C
```

So the shortest notation for an action of inserting "a" is

```
  ?a
```

## Simple example of titlecase

Here is a simple example of an input method that works as titlecase.

```
(input-method en titlecase)
(description (_ "Titlecase letters"))
(title "abc->Abc")
(map
  (toupper ("a" "A") ("b" "B") ("c" "C") ("d" "D") ("e" "E")
           ("f" "F") ("g" "G") ("h" "H") ("i" "I") ("j" "J")
           ("k" "K") ("l" "L") ("m" "M") ("n" "N") ("o" "O")
           ("p" "P") ("q" "Q") ("r" "R") ("s" "S") ("t" "T")
           ("u" "U") ("v" "V") ("w" "W") ("x" "X") ("y" "Y")
           ("z" "Z") ("ii" "İ"))
  (lower ("a" "a") ("b" "b") ("c" "c") ("d" "d") ("e" "e")
         ("f" "f") ("g" "g") ("h" "h") ("i" "i") ("j" "j")
         ("k" "k") ("l" "l") ("m" "m") ("n" "n") ("o" "o")
         ("p" "p") ("q" "q") ("r" "r") ("s" "s") ("t" "t")
         ("u" "u") ("v" "v") ("w" "w") ("x" "x") ("y" "y")
         ("z" "z")))
(state
  (init
    (toupper (shift non-upcase)))
  (non-upcase
    (lower (commit))
    (nil (shift init))))
```

The map name `nil` means that it matches with any key event that does not match any rules in the other maps in the current state. In addition, it does not consume any key event.

When you type "a quick blown fox" with this input method, you get "A Quick Blown Fox". OK, you find a typo in "blown", which should be "brown". To correct it, you probably move the cursor after "l" and type <Backspace> and <r>. However, if the current input method is still active, a capital "R" is inserted. It is not a sophisticated behavior.

## Surrounding text support

To make the input method work well also in such a case, we must use "surrounding text support". It is a way to check characters around the inputting spot and delete them if necessary. Note that this facility is available only with Gtk+ applications and Qt applications. You cannot use it with applications that use XIM to communicate with an input method.

Before explaining how to utilize "surrounding text support", you must understand how to use variables, arithmetic comparisons, and conditional actions.

At first, any symbol (except for several preserved ones) used as ARG of an action is treated as a variable. For instance, the commands

```
  (set X 32) (insert X)
```

set the variable `X` to integer value 32, then insert a character whose Unicode character code is 32 (i.e. SPACE).

The second argument of the `set` action can be an expression of this form:

```
  (OPERATOR ARG1 [ARG2])
```

Both ARG1 and ARG2 can be an expression. So,

```
  (set X (+ (* Y 32) Z))
```

sets `X` to the value of `Y * 32 + Z`.

We have the following arithmetic/bitwise OPERATORs (require two arguments):

```
  + - * / & |
```

these relational OPERATORs (require two arguments):

```
  == <= >= < >
```

and this logical OPERATOR (requires one argument):

```
  !
```

For surrounding text support, we have these preserved variables:

```
  @-0, @-N, @+N (N is a positive integer)
```

The values of them are predefined as below and can not be altered.

+   `-0`
    
    \-1 if surrounding text is supported, -2 if not.
    
+   `-N`
    
    The Nth previous character in the preedit buffer. If there are only M (M<N) previous characters in it, the value is the (N-M)th previous character from the inputting spot.
    
+   `+N`
    
    The Nth following character in the preedit buffer. If there are only M (M<N) following characters in it, the value is the (N-M)th following character from the inputting spot.
    

So, provided that you have this context:

```
  ABC|def|GHI
```

("def" is in the preedit buffer, two "|"s indicate borders between the preedit buffer and the surrounding text) and your current position in the preedit buffer is between "d" and "e", you get these values:

```
  @-3 -- ?B
  @-2 -- ?C
  @-1 -- ?d
  @+1 -- ?e
  @+2 -- ?f
  @+3 -- ?G
```

Next, you have to understand the conditional action of this form:

```
  (cond
    (EXPR1 ACTION ACTION ...)
    (EXPR2 ACTION ACTION ...)
    ...)
```

where EXPRn are expressions. When an input method executes this action, it resolves the values of EXPRn one by one from the first branch. If the value of EXPRn is resolved into nonzero, the corresponding actions are executed.

Now you are ready to write a new version of the input method "Titlecase".

```
(input-method en titlecase2)
(description (_ "Titlecase letters"))
(title "abc->Abc")
(map
  (toupper ("a" "A") ("b" "B") ("c" "C") ("d" "D") ("e" "E")
           ("f" "F") ("g" "G") ("h" "H") ("i" "I") ("j" "J")
           ("k" "K") ("l" "L") ("m" "M") ("n" "N") ("o" "O")
           ("p" "P") ("q" "Q") ("r" "R") ("s" "S") ("t" "T")
           ("u" "U") ("v" "V") ("w" "W") ("x" "X") ("y" "Y")
           ("z" "Z") ("ii" "İ")))
(state
  (init
    (toupper

     ;; Now we have exactly one uppercase character in the preedit
     ;; buffer.  So, "@-2" is the character just before the inputting
     ;; spot.

     (cond ((| (& (>= @-2 ?A) (<= @-2 ?Z))
               (& (>= @-2 ?a) (<= @-2 ?z))
               (= @-2 ?İ))

        ;; If the character before the inputting spot is A..Z,
        ;; a..z, or İ, remember the only character in the preedit
        ;; buffer in the variable X and delete it.

        (set X @-1) (delete @-)

        ;; Then insert the lowercase version of X.

        (cond ((= X ?İ) "i") 
                  (1 (set X (+ X 32)) (insert X))))))))
```

The above example contains the new action `delete`. So, it is time to explain more about the preedit buffer. The preedit buffer is a temporary place to store a sequence of characters. In this buffer, the input method keeps a position called the "current position". The current position exists between two characters, at the beginning of the buffer, or at the end of the buffer. The `insert` action inserts characters before the current position. For instance, when your preedit buffer contains "ab.c" ("." indicates the current position),

```
  (insert "xyz")
```

changes the buffer to "abxyz.c".

There are several predefined variables that represent a specific position in the preedit buffer. They are:

+   `@<, @=, @>`
    
    The first, current, and last positions.
    
+   `@-, @+`
    
    The previous and the next positions.
    

The format of the `delete` action is this:

```
  (delete POS)
```

where POS is a predefined positional variable. The above action deletes the characters between POS and the current position. So, `(delete -)` deletes one character before the current position. The other examples of `delete` include the followings:

```
  (delete @+)  ; delete the next character
  (delete @<)  ; delete all the preceding characters in the buffer
  (delete @>)  ; delete all the following characters in the buffer
```

You can change the current position using the `move` action as below:

```
  (move @-)  ; move the current position to the position before the
               previous character
  (move @<)  ; move to the first position
```

Other positional variables work similarly.

Let's see how our new example works. Whatever a key event is, the input method is in its only state, `init`. Since an event of a lower letter key is firstly handled by MAP-ACTIONs, every key is changed into the corresponding uppercase and put into the preedit buffer. Now this character can be accessed with `-1`.

How can we tell whether the new character should be a lowercase or an uppercase? We can do so by checking the character before it, i.e. `-2`. BRANCH-ACTIONs in the `init` state do the job.

It first checks if the character `-2` is between A to Z, between a to z, or İ by the conditional below.

```
     (cond ((| (& (>= @-2 ?A) (<= @-2 ?Z))
               (& (>= @-2 ?a) (<= @-2 ?z))
               (= @-2 ?İ))
```

If not, there is nothing to do specially. If so, our new key should be changed back into lowercase. Since the uppercase character is already in the preedit buffer, we retrieve and remember it in the variable `X` by

```
    (set X @-1)
```

and then delete that character by

```
    (delete @-)
```

Lastly we re-insert the character in its lowercase form. The problem here is that "İ" must be changed into "i", so we need another conditional. The first branch

```
    ((= X ?İ) "i")
```

means that "if the character remembered in X is 'İ', 'i' is inserted".

The second branch

```
    (1 (set X (+ X 32)) (insert X))
```

starts with "1", which is always resolved into nonzero, so this branch is a catchall. Actions in this branch increase `X` by 32, then insert `X`. In other words, they change A...Z into a...z respectively and insert the resulting lowercase character into the preedit buffer. As the input method reaches the end of the BRANCH-ACTIONs, the character is committed.

This new input method always checks the character before the current position, so "A Quick Blown Fox" will be successfully fixed to "A Quick Brown Fox" by the key sequence <BackSpace> <r>.