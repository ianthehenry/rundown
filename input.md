Hello world, this is a ~~complicated~~ *very simple* example.

```python in
print(hello)
```

# More text.

    $ echo this is an indentented one
    this is an indentented one

lots of newlines after this:



```
okay nice
```

and with indented blocks:



    i am an indented block

Now there's another paragraph.

> now here's a quote

and then:

>     here's an indented code block in a quote

but yet:

> ```
> here's a fenced code quote
> ```

Alright.

- list one
- list two
- list there
  - nested
  - on multiple
    lines

alright. but:

- list item
  ```
  that contains code
  ```

okay. not bad. not bad.

Lastly we need to check indented code blocks:

  ```fence
  i am a fence, indentation level three
  ```

And mismatched indentation:

  ```fence
   i am a fence, indentation level three
 ```

 ```fence
   i am a fence, indentation level three
  ```

More than three:

````fence
i am a fence, indentation level three
````

Mismatched number of fence characters:

````fence
i am a fence, indentation level three
`````

okay now adjacent code blocks:

```hello
i am a code block. lots of white space:
```




```
i am another code block. no whitespace:
```
```
i am a third code block.
```

```
unclosed code


