# Image Black

A CLI tool for image dataset clean-up

1.  Choose action (count, convert, list, ..)
2.  Choose filters (filesize>10M, png)
3.  (Optional) Choose transforms (rgb, width=512)

**Warning** You should escape angle brackets, e.g. `filesize>10M` to`"filesize>10M"` for your shell.

```sh
./image-black [mode] [filter] [filter] .. [source dir]
./image-black convert [filter] [filter] .. to [target] [target] .. [source dir] [dest dir]
./image-black convert [filter] [filter] .. into [target] [target] .. [source dir]

./image-black convert to gray          dir1 dir2
./image-black convert jpg   to rgb     dir1 dir2
./image-black convert jpg into rgb     dir1
./image-black convert rgb jpg to png   dir1 dir2
./image-black convert "width<128" to long=512 png dir1 dir2
./image-black count channel .
./image-black count gray .
./image-black remove "filesize>10M" .
```

|         |      | channel         | format                 | filesize                   | dim                                             | quality (WIP)                 | aspect (WIP)         |
| ------- | ---- | --------------- | ---------------------- | -------------------------- | ----------------------------------------------- | ----------------------------- | -------------------- |
|         |      | rgb, rgba, gray | png, jpg (covers jpeg) | filesize, 50B, 300K, 10.5M | width, height, long, short, =, ==, >, <, >=, <= | q, >, >=, <, <=, =, ==, 1~100 | aspect, >, >=, <, <= |
| any     |      | any RGBA        | any png                |                            |                                                 |                               |                      |
| list    |      | list rgba       |                        |                            |                                                 | list q<80                     |                      |
| count   |      | count !rgb      | count format           | count "filesize>10M"       | count "short>=512"                              |                               |                      |
| remove  |      | remove gray     | remove !png !jpg       |                            | remove "width<512" height==100                  |                               | -                    |
| convert |      |                 | convert jpg into png   |                            | convert long=512                                |                               |                      |

## Incompletes

- For now, convert+dim only works with long=x or short=x.

## Usage

- You can prepend ! to represent `not`. (any !rgb, count !filesize<10M png)

### any

```
image-black any [conditions..] source_dir
```

Check if any file satisfies the condition.

If any, it stops.

### list

```
image-black list [conditions..] source_dir
```

Print the list of paths, each prefixe with `source_dir`.

### count

```
image-black count [conditions..] source_dir
image-black count [attr] source_dir
```

pyCount the files with the condition, or report the statistics for the attribute.

### remove

```
image-black remove [conditions..] source_dir
```

### convert

```
image-black convert [goals..] source_dir dest_dir
```

`source_dir==dest_dir`is allowed.

does not remove the original files.

filesize cannot be used.

dim inequality cannot be used.



## Example Pipeline for HQ Face images

1. Basic
   1. Crawl images
   2. `image-black remove "filesize>10M"` (overbloated)
   3. `image-black remove "short<512"` (too small)
   4. `image-black convert to rgb` or `image-black remove gray`  (default jpeg q = 75)
2. Crop: MTCNN
   1. Run MTCNN (it excludes overlapping faces / bbox touching the boundary)
   2. Save only when `short>=128`
3. NIMA



## Golang?

`go/main.go` is an incomplete go version. The problem with golang is that the standard image library doesn not distinguish between RGB and RGBA.
Thus one needs to use _opencv_ or _vips_, however, they do not provide a convinient method for reading the metadata (width, height, colortype) without loading the entire file.

`go/count_small.go` counts the number of files whose the size is <10M.

`go/count_large.go` counts the number of files whose the both dimension (wh) is greater than 512px.
