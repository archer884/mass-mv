# mmv

> a mass rename program

```shell
~/src/mass-mv master
❯ mmv "{{o}} (backup)" src/*.rs
src/main.rs
 -> src/main (backup).rs
src/paths.rs
 -> src/paths (backup).rs
src/rename.rs
 -> src/rename (backup).rs
src/template.rs
 -> src/template (backup).rs
Would rename 4 files
```

## Basic operation

mmv performs three kinds of operations:

1. Preview
2. Copy
3. Rename

The rename operation is protected by the `--force` flag and won't happen if you don't use it. Copying is triggered by the `--copy` flag. Running with neither flag will just preview the operation performed by either of the others.

## Rename templates

Basically, renaming a file can only change the *file stem.* The file's directory and extension will be retained, so don't worry about those.

A template may contain any arbitrary text and the following two placeholders:

- n
- o

The `n` placeholder will insert the file's number in the sequence. (Files are sorted in alphabetical order; I'll look into adding other sort options some other time). If this number needs to have some amount of leading zeroes, just use more `nnn`. `{{nn}}` will result in `01, 02, 03, ...`.

The `o` placeholder calls for the program to insert the file's original name. Similar to the `n` placeholder, `ooo` will insert the first three characters of the original name. I don't know precisely why you would want to do this, but you can. That said, `o` by itself will just insert the full name.

Templates must be enclosed in `{{}}` to be recognized.
