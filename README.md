# BashDoc

A tool for generating documentation/help menu for user defined bash functions.

## Syntax

### Example

```bash
#;
# cd()
# moves to given directory
# @param directory: folder to move to
# @return void
#"
cd() {
    cd $1
}
```

Outputs

![](./demo.png)

with lots of color!

### Global Delimiters

`START_DELIM = #;`

`END_DELIM = #"`

`PAR_DELIM = @param`

`RET_DELIM = @return`

`OPT_DELIM = # -`

`COMM_DELIM = #`

These can be modifed in the code to your preference.

## Install

```bash
git clone https://github.com/dustinknopoff/bashdoc
cd bashdoc
cargo install
```
