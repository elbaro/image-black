# Image Black

A toolbox for images clean-up

1. Choose action (count, convert, list)
2. Choose filters (filesize>10M, png)
3. (Optional) For convert, list what you want targets (rgb, width=512)

```sh
./image-black [mode] [filter] [filter] .. [source dir]
./image-black convert [filter] [filter] .. to [target] [target] .. [source dir] [dest dir]
./image-black convert [filter] [filter] .. into [target] [target] .. [source dir]

./image-black convert jpg   to rgb     dir1 dir2
./image-black convert jpg into rgb     dir1 dir2
./image-black convert rgb jpg to png dir1 dir2
./image-black convert width<128 to long=512 png dir1 dir2
./image-black count channel .
./image-black count gray .
./image-black remove filesize>10M .
./image-black move long<512 ./imgs ../imgs_small
```



|         | valid          | channel         | format                 | filesize                   | dim                                             | quality (WIP)                 | aspect               |
| ------- | -------------- | --------------- | ---------------------- | -------------------------- | ----------------------------------------------- | ----------------------------- | -------------------- |
|         | valid, invalid | rgb, rgba, gray | png, jpg (covers jpeg) | filesize, 50B, 300K, 10.5M | width, height, long, short, =, ==, >, <, >=, <= | q, >, >=, <, <=, =, ==, 1~100 | aspect, >, >=, <, <= |
| any     |                | any RGBA        | any png                |                            |                                                 |                               |                      |
| list    |                | list rgba       |                        |                            |                                                 | list q<80                     |                      |
| count   |                | count           | count format           | count filesize>10M         | count short>=512                                |                               |                      |
| remove  |                |                 |                        |                            | remove width<512 height==100                    |                               | -                    |
| convert |                |                 | convert  jpg           |                            | convert long=512                                |                               |                      |



## Usage

- Multiple conditions are combined with `and` operation.
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

Count the files with the condition, or report the statistics for the attribute.



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



### move

WIP.



### copy

WIP.
